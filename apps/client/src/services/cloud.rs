//! Cloud REST 客户端 — 对应 apps/client/src/services/cloudImpl.ts
//!
//! 所有 DTO 类型从 lol_web_protocol 共享 crate 引用。
//! 内部请求体类型（无对应协议类型）在本文件局部定义。

use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};
use std::{env, fs};

use lol_web_protocol::admin::AdminMetrics;
use lol_web_protocol::agent::{Agent, CreateAgentDto, UpdateAgentDto};
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::auth::{
    AuthToken, CodeLoginRequest, LoginRequest, RegisterRequest, ResetPasswordRequest, UserInfo,
};
use lol_web_protocol::essence::{BillingPlan, CheckInResult, EssenceTransaction, SubscribeRequest};
use lol_web_protocol::history::{GameHistorySummary, SavedAgentHistory};
use lol_web_protocol::match_::{Match, MatchEvent};
use lol_web_protocol::model_provider::{
    ModelProvider, ModelProviderInput, TestModelProviderInput, TestModelProviderResponse,
};
use lol_web_protocol::rank::{EloRating, RankEnqueueRequest, RankQueueEntry, Season};
use lol_web_protocol::room::{
    AddSlotRequest, CreateRoomRequest, JoinByCodeRequest, Room, RoomAgentSlot, RoomConstraints,
    StartRoomResponse,
};
use lol_web_protocol::scenario::{CreateScenarioDto, Scenario, UpdateScenarioDto};
use lol_web_protocol::spawn_preset::{
    CreateSpawnPresetDto, SpawnPreset, Team, UpdateSpawnPresetDto, Visibility,
};
use reqwest::{Client as HttpClient, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::runtime::Runtime;

// ── Error ──

#[derive(Debug, Clone)]
pub enum CloudError {
    /// 服务端返回的业务错误或网络错误
    Http(String),
    /// 401 鉴权失败，底层已清 token 并触发回调
    Unauthorized,
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudError::Http(msg) => write!(f, "{}", msg),
            CloudError::Unauthorized => write!(f, "鉴权失效"),
        }
    }
}

impl std::error::Error for CloudError {}

// ── 局部请求体（无对应协议类型） ──

#[derive(Serialize)]
struct ForkAgentBody {
    new_name: Option<String>,
}

#[derive(Serialize)]
struct UploadHistoryBody {
    histories: Vec<SavedAgentHistory>,
}

#[derive(Serialize)]
struct UpdateVisibilityBody {
    visibility: Visibility,
}

// ── Token 持久化 ──

const TOKEN_FILENAME: &str = "auth_token";

fn moon_lol_dir() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".moon-lol")
}

fn token_path() -> PathBuf {
    moon_lol_dir().join(TOKEN_FILENAME)
}

fn load_token() -> Option<String> {
    let path = token_path();
    fs::read_to_string(&path).ok().filter(|s| !s.is_empty())
}

fn save_token(token: &str) {
    let dir = moon_lol_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(token_path(), token);
}

fn clear_token_file() {
    let _ = fs::remove_file(token_path());
}

// ── 401 回调类型 ──

pub type UnauthorizedCallback = Arc<dyn Fn() + Send + Sync + 'static>;

// ── CloudClient ──

#[derive(Clone)]
pub struct CloudClient {
    base_url: String,
    http: HttpClient,
    token: Arc<RwLock<Option<String>>>,
    on_unauthorized: Arc<RwLock<Option<UnauthorizedCallback>>>,
}

/// 全局 tokio runtime：reqwest 的 send/text 必须在 tokio runtime 上下文内执行，
/// 而 client 的 UI 主线程与 gpui AsyncApp 的 executor 都不是 tokio。
fn tokio_runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("创建 tokio runtime 失败"))
}

impl CloudClient {
    /// 创建客户端。
    ///
    /// `base_url` 缺省时读取环境变量 `VITE_BASE_URL`，再缺省使用 `http://127.0.0.1:8080`。
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url
            .or_else(|| env::var("VITE_BASE_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:8000".into());
        let token = load_token();
        // reqwest Client 的内部连接任务由 tokio::spawn 驱动，必须在 tokio runtime 内创建。
        let http = tokio_runtime().block_on(async { HttpClient::new() });
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
            token: Arc::new(RwLock::new(token)),
            on_unauthorized: Arc::new(RwLock::new(None)),
        }
    }

    /// 在全局 tokio runtime 内执行一个请求 future，把结果桥回当前 async 上下文。
    ///
    /// client 的 UI 调用方（gpui AsyncApp）不是 tokio runtime，reqwest 的
    /// send()/text() 必须在 tokio 内跑；这里用 oneshot 把结果送回调用方。
    async fn run_on_tokio<T, F, Fut>(&self, f: F) -> Result<T, CloudError>
    where
        T: Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T, CloudError>> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio_runtime().spawn(async move {
            let _ = tx.send(f().await);
        });
        rx.await
            .map_err(|e| CloudError::Http(format!("请求任务被取消: {e}")))?
    }

    /// 注册 401 回调。收到 401 时自动清 token 并调用该回调。
    pub fn set_on_unauthorized(&self, cb: UnauthorizedCallback) {
        *self.on_unauthorized.write().unwrap() = Some(cb);
    }

    // ── 登录态（同步） ──

    pub fn is_authenticated(&self) -> bool {
        self.token.read().unwrap().is_some()
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.read().unwrap().clone()
    }

    pub fn logout(&self) {
        *self.token.write().unwrap() = None;
        clear_token_file();
    }

    // ── 内部 HTTP 基础设施 ──

    fn trigger_unauthorized(&self) {
        if let Some(cb) = self.on_unauthorized.read().unwrap().as_ref() {
            cb();
        }
    }

    fn clear_auth(&self) {
        *self.token.write().unwrap() = None;
        clear_token_file();
    }

    fn store_token(&self, t: &str) {
        *self.token.write().unwrap() = Some(t.to_string());
        save_token(t);
    }

    /// 发送 HTTP 请求并读取响应文本，全部在全局 tokio runtime 内执行。
    /// 返回 (status, text)。401 在非 auth 路径上触发拦截。
    async fn do_request(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<(StatusCode, String), CloudError> {
        let url = format!("{}{}", self.base_url, path);
        let http = self.http.clone();
        let token = self.token.read().unwrap().clone();

        let result = self
            .run_on_tokio(move || async move {
                let mut req = http.request(method, &url);
                if let Some(t) = &token {
                    req = req.header("Authorization", format!("Bearer {}", t));
                }
                if let Some(b) = body {
                    req = req.header("Content-Type", "application/json").body(b);
                }

                let resp = req
                    .send()
                    .await
                    .map_err(|e| CloudError::Http(e.to_string()))?;
                let status = resp.status();
                let text = resp
                    .text()
                    .await
                    .map_err(|e| CloudError::Http(e.to_string()))?;
                Ok((status, text))
            })
            .await?;

        let (status, text) = result;
        if status == StatusCode::UNAUTHORIZED && !path.starts_with("/api/auth/") {
            self.clear_auth();
            self.trigger_unauthorized();
            return Err(CloudError::Unauthorized);
        }
        Ok((status, text))
    }

    /// 从响应信封中提取 `data` 字段并反序列化。
    fn parse_json<T: DeserializeOwned>(status: StatusCode, text: String) -> Result<T, CloudError> {
        let envelope: Value = if text.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&text).map_err(|e| CloudError::Http(e.to_string()))?
        };

        if !status.is_success() {
            let msg = envelope
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("未知错误");
            return Err(CloudError::Http(msg.to_string()));
        }

        let data = envelope.get("data").cloned().unwrap_or(Value::Null);
        serde_json::from_value(data).map_err(|e| CloudError::Http(e.to_string()))
    }

    /// 仅校验响应成功，不解析 data。
    fn parse_empty(status: StatusCode, text: String) -> Result<(), CloudError> {
        if status.is_success() {
            return Ok(());
        }

        let envelope: Value = if text.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&text).map_err(|e| CloudError::Http(e.to_string()))?
        };

        let msg = envelope
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("未知错误");
        Err(CloudError::Http(msg.to_string()))
    }

    // ── 便捷方法 ──

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, CloudError> {
        let (status, text) = self.do_request(Method::GET, path, None).await?;
        Self::parse_json(status, text)
    }

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CloudError> {
        let bytes = serde_json::to_vec(body).map_err(|e| CloudError::Http(e.to_string()))?;
        let (status, text) = self.do_request(Method::POST, path, Some(bytes)).await?;
        Self::parse_json(status, text)
    }

    async fn post_no_body_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, CloudError> {
        let (status, text) = self.do_request(Method::POST, path, None).await?;
        Self::parse_json(status, text)
    }

    async fn post_no_body_empty(&self, path: &str) -> Result<(), CloudError> {
        let (status, text) = self.do_request(Method::POST, path, None).await?;
        Self::parse_empty(status, text)
    }

    async fn post_empty<B: Serialize>(&self, path: &str, body: &B) -> Result<(), CloudError> {
        let bytes = serde_json::to_vec(body).map_err(|e| CloudError::Http(e.to_string()))?;
        let (status, text) = self.do_request(Method::POST, path, Some(bytes)).await?;
        Self::parse_empty(status, text)
    }

    async fn put_json<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CloudError> {
        let bytes = serde_json::to_vec(body).map_err(|e| CloudError::Http(e.to_string()))?;
        let (status, text) = self.do_request(Method::PUT, path, Some(bytes)).await?;
        Self::parse_json(status, text)
    }

    async fn put_empty<B: Serialize>(&self, path: &str, body: &B) -> Result<(), CloudError> {
        let bytes = serde_json::to_vec(body).map_err(|e| CloudError::Http(e.to_string()))?;
        let (status, text) = self.do_request(Method::PUT, path, Some(bytes)).await?;
        Self::parse_empty(status, text)
    }

    async fn patch_empty<B: Serialize>(&self, path: &str, body: &B) -> Result<(), CloudError> {
        let bytes = serde_json::to_vec(body).map_err(|e| CloudError::Http(e.to_string()))?;
        let (status, text) = self.do_request(Method::PATCH, path, Some(bytes)).await?;
        Self::parse_empty(status, text)
    }

    async fn delete_empty(&self, path: &str) -> Result<(), CloudError> {
        let (status, text) = self.do_request(Method::DELETE, path, None).await?;
        Self::parse_empty(status, text)
    }

    // ── Auth ──

    pub async fn login(&self, phone: &str, password: &str) -> Result<AuthToken, CloudError> {
        let body = LoginRequest {
            phone: phone.to_string(),
            password: password.to_string(),
        };
        let res = self
            .post_json::<AuthToken, _>("/api/auth/login", &body)
            .await?;
        self.store_token(&res.token);
        Ok(res)
    }

    pub async fn code_login(&self, phone: &str, code: &str) -> Result<AuthToken, CloudError> {
        let body = CodeLoginRequest {
            phone: phone.to_string(),
            code: code.to_string(),
        };
        let res = self
            .post_json::<AuthToken, _>("/api/auth/code-login", &body)
            .await?;
        self.store_token(&res.token);
        Ok(res)
    }

    pub async fn register(
        &self,
        phone: &str,
        password: &str,
        code: &str,
    ) -> Result<AuthToken, CloudError> {
        let body = RegisterRequest {
            phone: phone.to_string(),
            password: password.to_string(),
            code: code.to_string(),
        };
        let res = self
            .post_json::<AuthToken, _>("/api/auth/register", &body)
            .await?;
        self.store_token(&res.token);
        Ok(res)
    }

    pub async fn reset_password(
        &self,
        phone: &str,
        code: &str,
        new_password: &str,
    ) -> Result<(), CloudError> {
        let body = ResetPasswordRequest {
            phone: phone.to_string(),
            code: code.to_string(),
            new_password: new_password.to_string(),
        };
        self.post_empty("/api/auth/reset-password", &body).await
    }

    pub async fn get_current_user(&self) -> Result<UserInfo, CloudError> {
        self.get_json("/api/auth/me").await
    }

    // ── Model Providers ──

    pub async fn list_model_providers(&self) -> Result<Vec<ModelProvider>, CloudError> {
        self.get_json("/api/model-providers").await
    }

    pub async fn create_model_provider(
        &self,
        input: &ModelProviderInput,
    ) -> Result<ModelProvider, CloudError> {
        self.post_json("/api/model-providers", input).await
    }

    pub async fn update_model_provider(
        &self,
        id: &str,
        input: &ModelProviderInput,
    ) -> Result<(), CloudError> {
        let path = format!("/api/model-providers/{}", id);
        self.put_empty(&path, input).await
    }

    pub async fn delete_model_provider(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/model-providers/{}", id);
        self.delete_empty(&path).await
    }

    pub async fn test_model_provider(
        &self,
        input: &TestModelProviderInput,
    ) -> Result<TestModelProviderResponse, CloudError> {
        self.post_json("/api/model-providers/test", input).await
    }

    // ── Platform Models ──

    pub async fn list_platform_models(&self) -> Result<Vec<String>, CloudError> {
        self.get_json("/api/platform-models").await
    }

    // ── Agents ──

    pub async fn list_agents(&self) -> Result<Vec<Agent>, CloudError> {
        self.get_json("/api/agents").await
    }

    pub async fn get_agent(&self, id: &str) -> Result<Agent, CloudError> {
        let path = format!("/api/agents/{}", id);
        self.get_json(&path).await
    }

    pub async fn create_agent(&self, data: &CreateAgentDto) -> Result<Agent, CloudError> {
        self.post_json("/api/agents", data).await
    }

    pub async fn update_agent(&self, id: &str, data: &UpdateAgentDto) -> Result<Agent, CloudError> {
        let path = format!("/api/agents/{}", id);
        self.put_json(&path, data).await
    }

    pub async fn delete_agent(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/agents/{}", id);
        self.delete_empty(&path).await
    }

    pub async fn update_agent_visibility(
        &self,
        id: &str,
        visibility: Visibility,
    ) -> Result<(), CloudError> {
        let path = format!("/api/agents/{}/visibility", id);
        let body = UpdateVisibilityBody { visibility };
        self.patch_empty(&path, &body).await
    }

    // ── Agent Snapshots ──

    pub async fn publish_snapshot(&self, agent_id: &str) -> Result<AgentSnapshot, CloudError> {
        let path = format!("/api/agents/{}/publish", agent_id);
        self.post_no_body_json(&path).await
    }

    pub async fn list_snapshots(&self, agent_id: &str) -> Result<Vec<AgentSnapshot>, CloudError> {
        let path = format!("/api/agents/{}/snapshots", agent_id);
        self.get_json(&path).await
    }

    // ── Community ──

    pub async fn browse_community_agents(
        &self,
        sort: &str,
        limit: u32,
    ) -> Result<Vec<Agent>, CloudError> {
        let path = format!("/api/agents/community?sort={}&limit={}", sort, limit);
        self.get_json(&path).await
    }

    pub async fn fork_agent(
        &self,
        agent_id: &str,
        new_name: Option<&str>,
    ) -> Result<Agent, CloudError> {
        let path = format!("/api/agents/{}/fork", agent_id);
        let body = ForkAgentBody {
            new_name: new_name.map(|s| s.to_string()),
        };
        self.post_json(&path, &body).await
    }

    pub async fn pull_upstream(&self, agent_id: &str) -> Result<Agent, CloudError> {
        let path = format!("/api/agents/{}/pull-upstream", agent_id);
        self.post_no_body_json(&path).await
    }

    // ── Spawn Presets ──

    pub async fn list_spawn_presets(&self) -> Result<Vec<SpawnPreset>, CloudError> {
        self.get_json("/api/spawn-presets").await
    }

    pub async fn create_spawn_preset(
        &self,
        data: &CreateSpawnPresetDto,
    ) -> Result<SpawnPreset, CloudError> {
        self.post_json("/api/spawn-presets", data).await
    }

    pub async fn update_spawn_preset(
        &self,
        id: &str,
        data: &UpdateSpawnPresetDto,
    ) -> Result<SpawnPreset, CloudError> {
        let path = format!("/api/spawn-presets/{}", id);
        self.put_json(&path, data).await
    }

    pub async fn delete_spawn_preset(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/spawn-presets/{}", id);
        self.delete_empty(&path).await
    }

    // ── Scenarios ──

    pub async fn list_scenarios(&self) -> Result<Vec<Scenario>, CloudError> {
        self.get_json("/api/scenarios").await
    }

    pub async fn get_scenario(&self, id: &str) -> Result<Scenario, CloudError> {
        let path = format!("/api/scenarios/{}", id);
        self.get_json(&path).await
    }

    pub async fn create_scenario(&self, data: &CreateScenarioDto) -> Result<Scenario, CloudError> {
        self.post_json("/api/scenarios", data).await
    }

    pub async fn update_scenario(
        &self,
        id: &str,
        data: &UpdateScenarioDto,
    ) -> Result<Scenario, CloudError> {
        let path = format!("/api/scenarios/{}", id);
        self.put_json(&path, data).await
    }

    pub async fn delete_scenario(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/scenarios/{}", id);
        self.delete_empty(&path).await
    }

    pub async fn get_scenario_win_condition(&self, id: &str) -> Result<Option<Value>, CloudError> {
        let path = format!("/api/scenarios/{}/win-condition", id);
        self.get_json(&path).await
    }

    pub async fn set_scenario_win_condition(
        &self,
        id: &str,
        condition: &Value,
    ) -> Result<(), CloudError> {
        let path = format!("/api/scenarios/{}/win-condition", id);
        self.put_empty(&path, condition).await
    }

    // ── Game Histories ──

    pub async fn list_game_histories(&self) -> Result<Vec<GameHistorySummary>, CloudError> {
        self.get_json("/api/histories").await
    }

    pub async fn get_game_history_detail(
        &self,
        id: &str,
    ) -> Result<Vec<SavedAgentHistory>, CloudError> {
        let path = format!("/api/histories/{}", id);
        self.get_json(&path).await
    }

    pub async fn upload_game_history(
        &self,
        histories: Vec<SavedAgentHistory>,
    ) -> Result<(), CloudError> {
        let body = UploadHistoryBody { histories };
        self.post_empty("/api/histories", &body).await
    }

    pub async fn delete_game_history(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/histories/{}", id);
        self.delete_empty(&path).await
    }

    // ── Rooms ──

    pub async fn list_my_rooms(&self) -> Result<Vec<Room>, CloudError> {
        self.get_json("/api/rooms").await
    }

    pub async fn list_lobby_rooms(&self) -> Result<Vec<Room>, CloudError> {
        self.get_json("/api/rooms/lobby").await
    }

    pub async fn get_room(&self, id: &str) -> Result<Room, CloudError> {
        let path = format!("/api/rooms/{}", id);
        self.get_json(&path).await
    }

    pub async fn create_room(
        &self,
        name: &str,
        constraints: &RoomConstraints,
    ) -> Result<Room, CloudError> {
        let body = CreateRoomRequest {
            name: name.to_string(),
            constraints: *constraints,
        };
        self.post_json("/api/rooms", &body).await
    }

    pub async fn join_room(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/rooms/{}/join", id);
        self.post_no_body_empty(&path).await
    }

    pub async fn join_room_by_code(&self, code: &str) -> Result<Room, CloudError> {
        let body = JoinByCodeRequest {
            code: code.to_string(),
        };
        self.post_json("/api/rooms/join-by-code", &body).await
    }

    pub async fn leave_room(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/rooms/{}/leave", id);
        self.post_no_body_empty(&path).await
    }

    /// 解散房间（房主）
    pub async fn dissolve_room(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/rooms/{}", id);
        self.delete_empty(&path).await
    }

    pub async fn update_room_constraints(
        &self,
        id: &str,
        constraints: &RoomConstraints,
    ) -> Result<(), CloudError> {
        let path = format!("/api/rooms/{}", id);
        self.patch_empty(&path, constraints).await
    }

    pub async fn list_room_slots(&self, room_id: &str) -> Result<Vec<RoomAgentSlot>, CloudError> {
        let path = format!("/api/rooms/{}/agents", room_id);
        self.get_json(&path).await
    }

    pub async fn add_room_slot(
        &self,
        room_id: &str,
        agent_id: &str,
        team: Team,
    ) -> Result<RoomAgentSlot, CloudError> {
        let path = format!("/api/rooms/{}/agents", room_id);
        let body = AddSlotRequest {
            agent_id: agent_id
                .parse::<uuid::Uuid>()
                .map_err(|e| CloudError::Http(e.to_string()))?,
            team,
        };
        self.post_json(&path, &body).await
    }

    pub async fn remove_room_slot(&self, room_id: &str, slot_id: &str) -> Result<(), CloudError> {
        let path = format!("/api/rooms/{}/agents/{}", room_id, slot_id);
        self.delete_empty(&path).await
    }

    pub async fn start_room_match(&self, room_id: &str) -> Result<StartRoomResponse, CloudError> {
        let path = format!("/api/rooms/{}/start", room_id);
        self.post_no_body_json(&path).await
    }

    // ── Matches ──

    pub async fn list_my_matches(&self) -> Result<Vec<Match>, CloudError> {
        self.get_json("/api/matches").await
    }

    pub async fn get_match(&self, id: &str) -> Result<Match, CloudError> {
        let path = format!("/api/matches/{}", id);
        self.get_json(&path).await
    }

    pub async fn get_match_events(
        &self,
        id: &str,
        from_seq: u32,
        limit: u32,
    ) -> Result<Vec<MatchEvent>, CloudError> {
        let path = format!(
            "/api/matches/{}/events?from_seq={}&limit={}",
            id, from_seq, limit
        );
        self.get_json(&path).await
    }

    pub async fn stop_match(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/matches/{}/stop", id);
        self.post_no_body_empty(&path).await
    }

    // ── Rank ──

    pub async fn enqueue_rank(
        &self,
        agent_id: &str,
        snapshot_id: &str,
        mode: &str,
    ) -> Result<RankQueueEntry, CloudError> {
        let body = RankEnqueueRequest {
            agent_id: agent_id
                .parse::<uuid::Uuid>()
                .map_err(|e| CloudError::Http(e.to_string()))?,
            agent_snapshot_id: snapshot_id
                .parse::<uuid::Uuid>()
                .map_err(|e| CloudError::Http(e.to_string()))?,
            mode: mode.to_string(),
        };
        self.post_json("/api/rank/queue", &body).await
    }

    pub async fn get_rank_status(&self) -> Result<Vec<RankQueueEntry>, CloudError> {
        self.get_json("/api/rank/queue/status").await
    }

    pub async fn get_leaderboard(
        &self,
        mode: &str,
        limit: u32,
    ) -> Result<Vec<EloRating>, CloudError> {
        let path = format!("/api/rank/leaderboard?mode={}&limit={}", mode, limit);
        self.get_json(&path).await
    }

    pub async fn get_current_season(&self) -> Result<Season, CloudError> {
        self.get_json("/api/rank/seasons/current").await
    }

    // ── Essence ──

    pub async fn get_essence_balance(&self) -> Result<i64, CloudError> {
        self.get_json("/api/essence/balance").await
    }

    pub async fn check_in_essence(&self) -> Result<CheckInResult, CloudError> {
        self.post_no_body_json("/api/essence/check-in").await
    }

    pub async fn get_essence_transactions(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<EssenceTransaction>, CloudError> {
        let path = format!(
            "/api/essence/transactions?limit={}&offset={}",
            limit, offset
        );
        self.get_json(&path).await
    }

    // ── Subscription ──

    pub async fn get_current_subscription(&self) -> Result<BillingPlan, CloudError> {
        self.get_json("/api/subscriptions").await
    }

    pub async fn subscribe(&self, plan_id: &str) -> Result<(), CloudError> {
        let body = SubscribeRequest {
            plan_id: plan_id.to_string(),
        };
        self.post_empty("/api/subscriptions", &body).await
    }

    pub async fn list_billing_plans(&self) -> Result<Vec<BillingPlan>, CloudError> {
        self.get_json("/api/billing/plans").await
    }

    // ── Admin ──

    pub async fn get_admin_metrics(&self) -> Result<AdminMetrics, CloudError> {
        self.get_json("/api/admin/metrics").await
    }

    pub async fn list_running_matches(&self) -> Result<Vec<Match>, CloudError> {
        self.get_json("/api/admin/matches/running").await
    }

    pub async fn force_abort_match(&self, id: &str) -> Result<(), CloudError> {
        let path = format!("/api/admin/matches/{}/abort", id);
        self.post_no_body_empty(&path).await
    }
}

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use sqlx::postgres::PgPoolOptions;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

use crate::service::{InFrame, OutFrame, RLService};

const DEFAULT_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/moon_lol";

/// 启动 WebSocket 网络适配服务，将 WebSocket 连接绑定到 `RLService`
pub async fn start_rl_server(addr_str: &str) -> anyhow::Result<()> {
    let addr: SocketAddr = addr_str.parse()?;
    let listener = TcpListener::bind(&addr).await?;
    info!(
        "🚀 [lol_rl] RL WebSocket 传输层适配服务已成功启动在 ws://{}",
        addr
    );

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
    let pool_res = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(&db_url)
        .await;

    let (rl_service, _) = match pool_res {
        Ok(pool) => {
            info!("✅ [lol_rl] 成功连接至 PostgreSQL: {}", db_url);
            match RLService::new(pool, 100).await {
                Ok(srv) => srv,
                Err(e) => {
                    warn!(
                        "⚠️ [lol_rl] 初始化 PostgreSQL 架构失败 ({e})，降级为内存模式 (NoopRlRepo)"
                    );
                    RLService::new_in_memory(100).await
                }
            }
        }
        Err(e) => {
            warn!(
                "⚠️ [lol_rl] 无法连接到 PostgreSQL 数据库 ({e})。\n\
                 💡 当前已自动降级为【纯内存模式 (NoopRlRepo)】继续运行服务。\n\
                 💡 如需持久化保存训练任务、检查点与指标，请启动 PostgreSQL 并在环境变量中配置 DATABASE_URL。"
            );
            RLService::new_in_memory(100).await
        }
    };
    let service = Arc::new(rl_service);

    while let Ok((stream, peer)) = listener.accept().await {
        info!("🔗 [lol_rl] 新的 WebSocket 客户端连接来自: {}", peer);
        let service = service.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, service).await {
                warn!("⚠️ [lol_rl] 客户端连接异常断开: {e}");
            }
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, service: Arc<RLService>) -> anyhow::Result<()> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_writer, mut ws_reader) = ws_stream.split();
    let mut rx = service.subscribe();

    let init_bytes = bincode::serialize(&OutFrame::Status {
        task_id: "global".into(),
        status: "connected".into(),
    })?;
    ws_writer.send(Message::Binary(init_bytes.into())).await?;

    let send_task = tokio::spawn(async move {
        while let Ok(frame) = rx.recv().await {
            if let Ok(msg_bytes) = bincode::serialize(&frame) {
                if ws_writer
                    .send(Message::Binary(msg_bytes.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    while let Some(Ok(msg)) = ws_reader.next().await {
        match msg {
            Message::Binary(bytes) => {
                if let Ok(in_frame) = bincode::deserialize::<InFrame>(&bytes) {
                    service.handle_frame(in_frame).await;
                }
            }
            Message::Text(text) => {
                if let Ok(in_frame) = serde_json::from_str::<InFrame>(&text) {
                    service.handle_frame(in_frame).await;
                }
            }
            _ => {}
        }
    }

    send_task.abort();
    Ok(())
}

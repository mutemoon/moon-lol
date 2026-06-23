use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::interfaces::{
    ConfigService, PresetService, ScenarioService, GameService, HistoryService, LogService,
    UserService,
};
use crate::models::{
    AiConfig, SpawnPreset, AgentPreset, HeroPreset, FrontAgentConfig,
    GameConfig, QueryLogsParams, QueryLogsResult, LogRow
};

fn get_config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not find HOME or USERPROFILE environment variables".to_string())?;
    let dir = PathBuf::from(home).join(".moon-lol");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {e}"))?;
    }
    Ok(dir)
}

fn parse_legacy_env(content: &str) -> AiConfig {
    let mut api_key = String::new();
    let mut base_url = String::new();
    let mut preamble = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'').trim();
        if key == "ANTHROPIC_API_KEY" {
            api_key = val.to_string();
        } else if key == "ANTHROPIC_BASE_URL" {
            base_url = val.to_string();
        } else if key == "ANTHROPIC_PREAMBLE" {
            preamble = val.replace("\\n", "\n");
        }
    }

    AiConfig {
        api_key,
        base_url,
        preamble,
    }
}

// ── Database Initialization & Migration ──

pub async fn init_db(pool: &PgPool) -> Result<(), String> {
    // 1. Create users table first since it is independent
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            phone VARCHAR(20) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create users table: {e}"))?;

    // Check if column user_id exists in ai_config to decide if migration of schema is needed
    let has_user_id: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
            SELECT 1 
            FROM information_schema.columns 
            WHERE table_name='ai_config' AND column_name='user_id'
         );"
    )
    .fetch_one(pool)
    .await
    .unwrap_or((false,));

    if !has_user_id.0 {
        // Drop legacy tables to recreate them under isolated schema
        let _ = sqlx::query("DROP TABLE IF EXISTS ai_config").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS spawn_presets").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS agent_presets").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS hero_presets").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS custom_scenarios").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS scenario_win_conditions").execute(pool).await;
        let _ = sqlx::query("DROP TABLE IF EXISTS game_histories").execute(pool).await;
    }

    // 2. Create tables with user_id
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ai_config (
            user_id INT PRIMARY KEY,
            api_key TEXT NOT NULL,
            base_url TEXT NOT NULL,
            preamble TEXT NOT NULL
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create ai_config table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS spawn_presets (
            user_id INT NOT NULL,
            name TEXT NOT NULL,
            x REAL NOT NULL,
            z REAL NOT NULL,
            team TEXT NOT NULL,
            PRIMARY KEY (user_id, name)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create spawn_presets table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_presets (
            user_id INT NOT NULL,
            name TEXT NOT NULL,
            agent_type TEXT NOT NULL,
            prompt TEXT NOT NULL,
            preamble TEXT NOT NULL,
            model TEXT NOT NULL,
            PRIMARY KEY (user_id, name)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create agent_presets table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS hero_presets (
            user_id INT NOT NULL,
            name TEXT NOT NULL,
            champion TEXT NOT NULL,
            agent_preset_name TEXT NOT NULL,
            spawn_preset_name TEXT NOT NULL,
            PRIMARY KEY (user_id, name)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create hero_presets table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS custom_scenarios (
            user_id INT NOT NULL,
            name TEXT NOT NULL,
            agents JSONB NOT NULL,
            PRIMARY KEY (user_id, name)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create custom_scenarios table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS scenario_win_conditions (
            user_id INT NOT NULL,
            scene_name TEXT NOT NULL,
            condition JSONB NOT NULL,
            PRIMARY KEY (user_id, scene_name)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create scenario_win_conditions table: {e}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS game_histories (
            user_id INT NOT NULL,
            datetime TEXT NOT NULL,
            duration DOUBLE PRECISION NOT NULL,
            agents JSONB NOT NULL,
            details JSONB NOT NULL,
            PRIMARY KEY (user_id, datetime)
         );"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create game_histories table: {e}"))?;

    // 3. Perform automated migration from files if database is empty
    let _ = migrate_presets_to_postgres(pool).await;
    let _ = migrate_histories_to_postgres(pool).await;

    Ok(())
}

async fn migrate_presets_to_postgres(pool: &PgPool) -> Result<(), String> {
    let default_user_id = 1;
    // Spawn Presets
    if let Ok(path) = get_config_dir().map(|d| d.join("spawn_presets.json")) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(presets) = serde_json::from_str::<Vec<SpawnPreset>>(&content) {
                    for p in presets {
                        let _ = sqlx::query(
                            "INSERT INTO spawn_presets (user_id, name, x, z, team) VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (user_id, name) DO NOTHING"
                        )
                        .bind(default_user_id)
                        .bind(p.name)
                        .bind(p.x)
                        .bind(p.z)
                        .bind(p.team)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }
    }

    // Agent Presets
    if let Ok(path) = get_config_dir().map(|d| d.join("agent_presets.json")) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(presets) = serde_json::from_str::<Vec<AgentPreset>>(&content) {
                    for p in presets {
                        let _ = sqlx::query(
                            "INSERT INTO agent_presets (user_id, name, agent_type, prompt, preamble, model) VALUES ($1, $2, $3, $4, $5, $6)
                             ON CONFLICT (user_id, name) DO NOTHING"
                        )
                        .bind(default_user_id)
                        .bind(p.name)
                        .bind(p.agent_type)
                        .bind(p.prompt)
                        .bind(p.preamble)
                        .bind(p.model)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }
    }

    // Hero Presets
    if let Ok(path) = get_config_dir().map(|d| d.join("hero_presets.json")) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(presets) = serde_json::from_str::<Vec<HeroPreset>>(&content) {
                    for p in presets {
                        let _ = sqlx::query(
                            "INSERT INTO hero_presets (user_id, name, champion, agent_preset_name, spawn_preset_name) VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (user_id, name) DO NOTHING"
                        )
                        .bind(default_user_id)
                        .bind(p.name)
                        .bind(p.champion)
                        .bind(p.agent_preset_name)
                        .bind(p.spawn_preset_name)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }
    }

    // Custom Scenarios & Win Conditions
    if let Ok(dir) = get_config_dir().map(|d| d.join("games")) {
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() && p.extension().is_some_and(|ext| ext == "json") {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".win.json") {
                                let scene_name = name.trim_end_matches(".win.json");
                                if let Ok(content) = fs::read_to_string(&p) {
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                        let _ = sqlx::query(
                                            "INSERT INTO scenario_win_conditions (user_id, scene_name, condition) VALUES ($1, $2, $3)
                                             ON CONFLICT (user_id, scene_name) DO NOTHING"
                                        )
                                        .bind(default_user_id)
                                        .bind(scene_name)
                                        .bind(val)
                                        .execute(pool)
                                        .await;
                                    }
                                }
                                continue;
                            }
                        }
                        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(content) = fs::read_to_string(&p) {
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                    let _ = sqlx::query(
                                        "INSERT INTO custom_scenarios (user_id, name, agents) VALUES ($1, $2, $3)
                                         ON CONFLICT (user_id, name) DO NOTHING"
                                    )
                                    .bind(default_user_id)
                                    .bind(stem)
                                    .bind(val)
                                    .execute(pool)
                                    .await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn migrate_histories_to_postgres(pool: &PgPool) -> Result<(), String> {
    let default_user_id = 1;
    let history_dir = match get_config_dir() {
        Ok(d) => d.join("history"),
        Err(_) => return Ok(()),
    };
    if !history_dir.exists() {
        return Ok(());
    }

    let entries = match fs::read_dir(history_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(datetime_str) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !path.is_dir() {
            continue;
        }

        let exists = sqlx::query("SELECT 1 FROM game_histories WHERE user_id = $1 AND datetime = $2")
            .bind(default_user_id)
            .bind(datetime_str)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);

        if exists.is_some() {
            continue;
        }

        let files = match fs::read_dir(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mut agents = Vec::new();
        let mut details = Vec::new();
        let mut game_duration = 0.0;

        for sub_entry in files.flatten() {
            let sub_path = sub_entry.path();
            if !sub_path.is_file() || !sub_path.extension().is_some_and(|ext| ext == "json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&sub_path) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                    let agent_id = parsed.get("agent_id").and_then(|a| a.as_str()).unwrap_or("unknown").to_string();
                    let champion = parsed.get("champion").and_then(|c| c.as_str()).unwrap_or("unknown").to_string();
                    let team = parsed.get("team").and_then(|t| t.as_str()).unwrap_or("unknown").to_string();
                    let duration = parsed.get("game_duration").and_then(|d| d.as_f64()).unwrap_or(0.0);

                    agents.push(serde_json::json!({
                        "agent_id": agent_id,
                        "champion": champion,
                        "team": team,
                    }));
                    game_duration = duration;
                    details.push(parsed);
                }
            }
        }

        if !agents.is_empty() {
            let agents_json = serde_json::to_value(agents).unwrap_or(serde_json::Value::Null);
            let details_json = serde_json::to_value(details).unwrap_or(serde_json::Value::Null);

            let _ = sqlx::query(
                "INSERT INTO game_histories (user_id, datetime, duration, agents, details) VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (user_id, datetime) DO NOTHING"
            )
            .bind(default_user_id)
            .bind(datetime_str)
            .bind(game_duration)
            .bind(agents_json)
            .bind(details_json)
            .execute(pool)
            .await;
        }
    }

    Ok(())
}

// ── Config Service Implementation ──

pub struct ConfigServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_ai_config(&self, user_id: i32) -> Result<AiConfig, String> {
        let row = sqlx::query("SELECT api_key, base_url, preamble FROM ai_config WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            return Ok(AiConfig {
                api_key: r.try_get("api_key").map_err(|e| e.to_string())?,
                base_url: r.try_get("base_url").map_err(|e| e.to_string())?,
                preamble: r.try_get("preamble").map_err(|e| e.to_string())?,
            });
        }

        let dir = get_config_dir()?;
        let json_path = dir.join("config.json");
        let env_path = dir.join(".env");

        let config = if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| e.to_string())?;
            serde_json::from_str(&content).map_err(|e| e.to_string())?
        } else if env_path.exists() {
            let content = fs::read_to_string(&env_path).map_err(|e| e.to_string())?;
            parse_legacy_env(&content)
        } else {
            AiConfig {
                api_key: String::new(),
                base_url: String::new(),
                preamble: String::new(),
            }
        };

        let _ = sqlx::query(
            "INSERT INTO ai_config (user_id, api_key, base_url, preamble) VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id) DO UPDATE SET api_key = EXCLUDED.api_key, base_url = EXCLUDED.base_url, preamble = EXCLUDED.preamble"
        )
        .bind(user_id)
        .bind(&config.api_key)
        .bind(&config.base_url)
        .bind(&config.preamble)
        .execute(&self.pool)
        .await;

        Ok(config)
    }

    async fn set_ai_config(&self, user_id: i32, config: AiConfig) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO ai_config (user_id, api_key, base_url, preamble) VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id) DO UPDATE SET api_key = EXCLUDED.api_key, base_url = EXCLUDED.base_url, preamble = EXCLUDED.preamble"
        )
        .bind(user_id)
        .bind(&config.api_key)
        .bind(&config.base_url)
        .bind(&config.preamble)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        unsafe {
            std::env::set_var("ANTHROPIC_API_KEY", config.api_key.trim());
            std::env::set_var("ANTHROPIC_BASE_URL", config.base_url.trim());
            std::env::set_var("ANTHROPIC_PREAMBLE", config.preamble.trim());
        }

        Ok(())
    }
}

// ── Preset Service Implementation ──

pub struct PresetServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl PresetService for PresetServiceImpl {
    async fn list_spawn_presets(&self, user_id: i32) -> Result<Vec<SpawnPreset>, String> {
        let rows = sqlx::query("SELECT name, x, z, team FROM spawn_presets WHERE user_id = $1 ORDER BY name")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut presets = Vec::new();
        for r in rows {
            presets.push(SpawnPreset {
                name: r.try_get("name").map_err(|e| e.to_string())?,
                x: r.try_get("x").map_err(|e| e.to_string())?,
                z: r.try_get("z").map_err(|e| e.to_string())?,
                team: r.try_get("team").map_err(|e| e.to_string())?,
            });
        }
        Ok(presets)
    }

    async fn save_spawn_preset(&self, user_id: i32, preset: SpawnPreset) -> Result<(), String> {
        if preset.name.trim().is_empty() {
            return Err("Spawn preset name cannot be empty".to_string());
        }
        sqlx::query(
            "INSERT INTO spawn_presets (user_id, name, x, z, team) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (user_id, name) DO UPDATE SET x = EXCLUDED.x, z = EXCLUDED.z, team = EXCLUDED.team"
        )
        .bind(user_id)
        .bind(&preset.name)
        .bind(preset.x)
        .bind(preset.z)
        .bind(&preset.team)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn delete_spawn_preset(&self, user_id: i32, name: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM spawn_presets WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn list_agent_presets(&self, user_id: i32) -> Result<Vec<AgentPreset>, String> {
        let rows = sqlx::query("SELECT name, agent_type, prompt, preamble, model FROM agent_presets WHERE user_id = $1 ORDER BY name")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut presets = Vec::new();
        for r in rows {
            presets.push(AgentPreset {
                name: r.try_get("name").map_err(|e| e.to_string())?,
                agent_type: r.try_get("agent_type").map_err(|e| e.to_string())?,
                prompt: r.try_get("prompt").map_err(|e| e.to_string())?,
                preamble: r.try_get("preamble").map_err(|e| e.to_string())?,
                model: r.try_get("model").map_err(|e| e.to_string())?,
            });
        }
        Ok(presets)
    }

    async fn save_agent_preset(&self, user_id: i32, preset: AgentPreset) -> Result<(), String> {
        if preset.name.trim().is_empty() {
            return Err("Agent preset name cannot be empty".to_string());
        }
        sqlx::query(
            "INSERT INTO agent_presets (user_id, name, agent_type, prompt, preamble, model) VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id, name) DO UPDATE SET agent_type = EXCLUDED.agent_type, prompt = EXCLUDED.prompt, preamble = EXCLUDED.preamble, model = EXCLUDED.model"
        )
        .bind(user_id)
        .bind(&preset.name)
        .bind(&preset.agent_type)
        .bind(&preset.prompt)
        .bind(&preset.preamble)
        .bind(&preset.model)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn delete_agent_preset(&self, user_id: i32, name: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM agent_presets WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn list_hero_presets(&self, user_id: i32) -> Result<Vec<HeroPreset>, String> {
        let rows = sqlx::query("SELECT name, champion, agent_preset_name, spawn_preset_name FROM hero_presets WHERE user_id = $1 ORDER BY name")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut presets = Vec::new();
        for r in rows {
            presets.push(HeroPreset {
                name: r.try_get("name").map_err(|e| e.to_string())?,
                champion: r.try_get("champion").map_err(|e| e.to_string())?,
                agent_preset_name: r.try_get("agent_preset_name").map_err(|e| e.to_string())?,
                spawn_preset_name: r.try_get("spawn_preset_name").map_err(|e| e.to_string())?,
            });
        }
        Ok(presets)
    }

    async fn save_hero_preset(&self, user_id: i32, preset: HeroPreset) -> Result<(), String> {
        if preset.name.trim().is_empty() {
            return Err("Hero preset name cannot be empty".to_string());
        }
        sqlx::query(
            "INSERT INTO hero_presets (user_id, name, champion, agent_preset_name, spawn_preset_name) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (user_id, name) DO UPDATE SET champion = EXCLUDED.champion, agent_preset_name = EXCLUDED.agent_preset_name, spawn_preset_name = EXCLUDED.spawn_preset_name"
        )
        .bind(user_id)
        .bind(&preset.name)
        .bind(&preset.champion)
        .bind(&preset.agent_preset_name)
        .bind(&preset.spawn_preset_name)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn delete_hero_preset(&self, user_id: i32, name: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM hero_presets WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ── Scenario Service Implementation ──

pub struct ScenarioServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl ScenarioService for ScenarioServiceImpl {
    async fn list_custom_scenarios(&self, user_id: i32) -> Result<Vec<String>, String> {
        let rows = sqlx::query("SELECT name FROM custom_scenarios WHERE user_id = $1 ORDER BY name")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut scenarios = Vec::new();
        for r in rows {
            scenarios.push(r.try_get("name").map_err(|e| e.to_string())?);
        }
        Ok(scenarios)
    }

    async fn load_custom_scenario(&self, user_id: i32, scene_name: &str) -> Result<Vec<FrontAgentConfig>, String> {
        let row = sqlx::query("SELECT agents FROM custom_scenarios WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(scene_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let Some(r) = row else {
            return Err(format!("Scenario config not found: {scene_name}"));
        };

        let agents_val: serde_json::Value = r.try_get("agents").map_err(|e| e.to_string())?;
        let agents = serde_json::from_value(agents_val).map_err(|e| e.to_string())?;
        Ok(agents)
    }

    async fn save_custom_scenario(&self, user_id: i32, scene_name: &str, agents: Vec<FrontAgentConfig>) -> Result<(), String> {
        let mut resolved_agents = Vec::new();
        for (idx, agent) in agents.into_iter().enumerate() {
            let mut resolved = agent.clone();
            if resolved.id.is_none() {
                let champ_lower = agent.champion.to_lowercase();
                resolved.id = Some(format!("{champ_lower}_{idx}"));
            }
            resolved_agents.push(resolved);
        }

        let agents_json = serde_json::to_value(&resolved_agents).map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO custom_scenarios (user_id, name, agents) VALUES ($1, $2, $3)
             ON CONFLICT (user_id, name) DO UPDATE SET agents = EXCLUDED.agents"
        )
        .bind(user_id)
        .bind(scene_name)
        .bind(agents_json)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let dir = get_config_dir()?.join("games");
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        }

        let ron_path = dir.join(format!("{scene_name}.ron"));
        let mut ron_content = String::new();
        ron_content.push_str("(\n    resources: {},\n    entities: {\n");

        for (idx, agent) in resolved_agents.iter().enumerate() {
            let entity_id = 4294967185 + idx as u64;
            let x = agent.spawn_point[0];
            let z = agent.spawn_point[1];
            let y = if agent.champion == "Fiora" { 38.0 } else { 0.0 };
            let team = &agent.team;
            let champ_lower = agent.champion.to_lowercase();
            let agent_id = agent.id.as_ref().unwrap();

            ron_content.push_str(&format!("        {entity_id}: (\n"));
            ron_content.push_str("            components: {\n");
            ron_content.push_str("                \"bevy_transform::components::transform::Transform\": (\n");
            ron_content.push_str(&format!(
                "                    translation: ({:.1}, {:.1}, {:.1}),\n",
                x, y, z
            ));
            ron_content.push_str("                    rotation: (0.0, 0.0, 0.0, 1.0),\n");
            ron_content.push_str("                    scale: (1.0, 1.0, 1.0),\n");
            ron_content.push_str("                ),\n");
            ron_content.push_str(&format!(
                "                \"lol_core::team::Team\": {team},\n"
            ));
            ron_content.push_str(&format!(
                "                \"lol_champions::{champ_lower}::{}\": (),\n",
                agent.champion
            ));
            if idx == 0 {
                ron_content.push_str("                \"lol_render::controller::Controller\": (),\n");
                ron_content.push_str("                \"lol_render::controller::SelfPlayer\": (),\n");
                ron_content.push_str("                \"lol_render::camera::Focus\": (),\n");
            }
            ron_content.push_str("                \"lol_core::entities::champion::Champion\": (),\n");
            ron_content.push_str(&format!(
                "                \"lol_core::entities::champion::AgentId\": (\"{agent_id}\"),\n"
            ));
            ron_content.push_str("                \"lol_base::character::ConfigCharacterRecord\": (\n");
            ron_content.push_str(&format!(
                "                    character_record: Path(\"characters/{champ_lower}/config.ron\"),\n"
            ));
            ron_content.push_str("                ),\n");
            ron_content.push_str("                \"lol_base::character::ConfigSkin\": (\n");
            ron_content.push_str(&format!(
                "                    skin: Path(\"characters/{champ_lower}/skins/skin0.ron\"),\n"
            ));
            ron_content.push_str("                ),\n");
            ron_content.push_str("            },\n");
            ron_content.push_str("        ),\n");
        }

        ron_content.push_str("    },\n)\n");
        fs::write(&ron_path, ron_content).map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete_custom_scenario(&self, user_id: i32, scene_name: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM custom_scenarios WHERE user_id = $1 AND name = $2")
            .bind(user_id)
            .bind(scene_name)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let dir = get_config_dir()?.join("games");
        let json_path = dir.join(format!("{scene_name}.json"));
        let ron_path = dir.join(format!("{scene_name}.ron"));

        if json_path.exists() {
            let _ = fs::remove_file(json_path);
        }
        if ron_path.exists() {
            let _ = fs::remove_file(ron_path);
        }
        Ok(())
    }

    async fn load_scenario_win_condition(&self, user_id: i32, scene_name: &str) -> Result<serde_json::Value, String> {
        let row = sqlx::query("SELECT condition FROM scenario_win_conditions WHERE user_id = $1 AND scene_name = $2")
            .bind(user_id)
            .bind(scene_name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let condition: serde_json::Value = r.try_get("condition").map_err(|e| e.to_string())?;
            return Ok(condition);
        }
        Ok(serde_json::Value::Null)
    }

    async fn save_scenario_win_condition(&self, user_id: i32, scene_name: &str, condition: serde_json::Value) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO scenario_win_conditions (user_id, scene_name, condition) VALUES ($1, $2, $3)
             ON CONFLICT (user_id, scene_name) DO UPDATE SET condition = EXCLUDED.condition"
        )
        .bind(user_id)
        .bind(scene_name)
        .bind(condition)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ── Game Service Implementation ──

pub struct GameServiceImpl {
    inner: Mutex<GameInnerState>,
    workspace_root: PathBuf,
}

struct GameInnerState {
    bevy_process: Option<Child>,
    port: Option<u16>,
}

impl GameServiceImpl {
    pub fn new() -> Self {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let workspace_root = Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            inner: Mutex::new(GameInnerState {
                bevy_process: None,
                port: None,
            }),
            workspace_root,
        }
    }
}

#[async_trait]
impl GameService for GameServiceImpl {
    async fn start_game(&self, _user_id: i32, config: GameConfig) -> Result<(), String> {
        // We will read the config using std::env variables, or from file since ConfigServiceImpl requires PgPool.
        // As a fallback to avoid DB pool lifetime issues inside GameService (which doesn't hold PgPool),
        // we can read it from the `ai_config` table using a direct pool query, OR just use the filesystem
        // copy at `config.json` which is always kept in sync by `set_ai_config` or fallback to env.
        let config_data = get_config_dir().ok().and_then(|dir| {
            let json_path = dir.join("config.json");
            if json_path.exists() {
                if let Ok(content) = fs::read_to_string(&json_path) {
                    serde_json::from_str::<AiConfig>(&content).ok()
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut s = self.inner.lock().map_err(|e| e.to_string())?;

        if let Some(ref mut child) = s.bevy_process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    s.bevy_process = None;
                    s.port = None;
                }
                Ok(None) => return Err("Game already running".into()),
                Err(_) => {
                    s.bevy_process = None;
                    s.port = None;
                }
            }
        }

        let port = 9001;

        if let Some(config_data) = config_data {
            if !config_data.api_key.is_empty() {
                unsafe {
                    std::env::set_var("ANTHROPIC_API_KEY", config_data.api_key.trim());
                }
            }
            if !config_data.base_url.is_empty() {
                unsafe {
                    std::env::set_var("ANTHROPIC_BASE_URL", config_data.base_url.trim());
                }
            }
            if !config_data.preamble.is_empty() {
                unsafe {
                    std::env::set_var("ANTHROPIC_PREAMBLE", config_data.preamble.trim());
                }
            }
        }

        let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
            "info,lol_core=debug,lol_server=debug,lol_champions=debug,lol_render=debug,moon_lol=debug".to_string()
        });

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.workspace_root)
            .args(["run", "--"])
            .arg("--ws-port")
            .arg(port.to_string())
            .arg("--mode")
            .arg(&config.mode)
            .arg("--champion")
            .arg(&config.champion);

        if let Some(ref scene) = config.scene_name {
            cmd.arg("--scene")
                .arg(format!("user_games://{scene}.ron"));
        }

        let child = cmd.env("RUST_LOG", &rust_log)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn Bevy: {e}"))?;

        s.bevy_process = Some(child);
        s.port = Some(port);

        Ok(())
    }

    async fn stop_game(&self) -> Result<(), String> {
        let mut s = self.inner.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = s.bevy_process.take() {
            let _ = child.kill();
        }
        s.port = None;
        Ok(())
    }

    async fn get_active_game_port(&self) -> Result<Option<u16>, String> {
        let mut s = self.inner.lock().map_err(|e| e.to_string())?;
        if let Some(ref mut child) = s.bevy_process {
            if let Ok(Some(_)) = child.try_wait() {
                s.bevy_process = None;
                s.port = None;
            }
        }
        Ok(s.port)
    }
}

// ── History Service Implementation ──

pub struct HistoryServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl HistoryService for HistoryServiceImpl {
    async fn list_game_histories(&self, user_id: i32) -> Result<Vec<serde_json::Value>, String> {
        let rows = sqlx::query("SELECT datetime, duration, agents FROM game_histories WHERE user_id = $1 ORDER BY datetime DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut summaries = Vec::new();
        for r in rows {
            let datetime: String = r.try_get("datetime").map_err(|e| e.to_string())?;
            let duration: f64 = r.try_get("duration").map_err(|e| e.to_string())?;
            let agents: serde_json::Value = r.try_get("agents").map_err(|e| e.to_string())?;
            summaries.push(serde_json::json!({
                "datetime": datetime,
                "duration": duration,
                "agents": agents,
            }));
        }
        Ok(summaries)
    }

    async fn get_game_history_detail(&self, user_id: i32, datetime: &str) -> Result<Vec<serde_json::Value>, String> {
        let row = sqlx::query("SELECT details FROM game_histories WHERE user_id = $1 AND datetime = $2")
            .bind(user_id)
            .bind(datetime)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(r) = row {
            let details_val: serde_json::Value = r.try_get("details").map_err(|e| e.to_string())?;
            if let serde_json::Value::Array(arr) = details_val {
                return Ok(arr);
            }
        }
        Err("Game history record not found".to_string())
    }

    async fn delete_game_history(&self, user_id: i32, datetime: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM game_histories WHERE user_id = $1 AND datetime = $2")
            .bind(user_id)
            .bind(datetime)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ── Log Service Implementation ──
//
// 说明：web server 是远程服务，生产环境通常没有本地 Bevy 的 debug.db 可读。
// 此实现保留是为了迁移期兼容；后续新架构中日志读取会改为按 match_id 从
// match_events 表（Postgres）查询，或经 Bevy 进程的 WS 拉取。

pub struct LogServiceImpl;

fn log_db_path() -> Result<PathBuf, String> {
    let base = get_config_dir()?;
    let path = base.join("logs").join("debug.db");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

/// 以只读模式打开 SQLite 日志库（sqlx async，与 lol_core 的 writer 共享同一文件）。
async fn open_log_db_readonly(path: &PathBuf) -> Result<sqlx::SqlitePool, String> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .immutable(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())
}

#[async_trait]
impl LogService for LogServiceImpl {
    async fn query_log_entities(&self, _user_id: i32) -> Result<Vec<serde_json::Value>, String> {
        let db_path = log_db_path()?;
        if !db_path.exists() {
            return Ok(Vec::new());
        }
        let pool = open_log_db_readonly(&db_path).await?;

        let rows = sqlx::query(
            "SELECT DISTINCT entity_id, entity_name FROM logs \
             WHERE entity_id IS NOT NULL ORDER BY entity_id",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(serde_json::json!({
                "entity_id": row.try_get::<Option<i64>, _>(0).map_err(|e| e.to_string())?,
                "entity_name": row.try_get::<Option<String>, _>(1).map_err(|e| e.to_string())?,
            }));
        }
        Ok(result)
    }

    async fn query_log_categories(&self, _user_id: i32) -> Result<Vec<serde_json::Value>, String> {
        let db_path = log_db_path()?;
        if !db_path.exists() {
            return Ok(Vec::new());
        }
        let pool = open_log_db_readonly(&db_path).await?;

        let rows = sqlx::query(
            "SELECT DISTINCT category FROM logs WHERE category IS NOT NULL ORDER BY category",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(serde_json::json!({
                "category": row.try_get::<Option<String>, _>(0).map_err(|e| e.to_string())?,
            }));
        }
        Ok(result)
    }

    async fn query_logs(
        &self,
        _user_id: i32,
        params: QueryLogsParams,
    ) -> Result<QueryLogsResult, String> {
        let db_path = log_db_path()?;
        if !db_path.exists() {
            return Ok(QueryLogsResult {
                rows: Vec::new(),
                total_count: 0,
            });
        }
        let pool = open_log_db_readonly(&db_path).await?;

        let limit = params.limit.clamp(0, 1000) as i64;

        // ── 构造 WHERE 子句与绑定参数（全部走 bind，不内联，防注入）──
        let mut where_clause = String::from("WHERE 1=1");
        let mut levels_args: Vec<String> = vec![];
        let mut entity_id_arg: Option<i64> = None;
        let mut category_arg: Option<String> = None;
        let mut search_arg: Option<String> = None;

        if let Some(ref lvl) = params.levels {
            if !lvl.is_empty() {
                let placeholders: Vec<&str> = lvl.iter().map(|_| "?").collect();
                where_clause.push_str(&format!(" AND level IN ({})", placeholders.join(",")));
                levels_args = lvl.clone();
            }
        }
        if let Some(eid) = params.entity_id {
            where_clause.push_str(" AND entity_id = ?");
            entity_id_arg = Some(eid as i64);
        }
        if let Some(ref cat) = params.category {
            where_clause.push_str(" AND category = ?");
            category_arg = Some(cat.clone());
        }
        if let Some(ref search) = params.search_text {
            let search = search.trim();
            if !search.is_empty() {
                where_clause.push_str(" AND message LIKE ?");
                search_arg = Some(format!("%{search}%"));
            }
        }

        // 1. COUNT
        let sql_count = format!("SELECT COUNT(*) FROM logs {where_clause}");
        let total_count: i64 = {
            let mut q = sqlx::query_scalar::<_, i64>(&sql_count);
            for l in &levels_args {
                q = q.bind(l);
            }
            if let Some(eid) = entity_id_arg {
                q = q.bind(eid);
            }
            if let Some(ref cat) = category_arg {
                q = q.bind(cat);
            }
            if let Some(ref search) = search_arg {
                q = q.bind(search);
            }
            q.fetch_one(&pool)
                .await
                .map_err(|e| e.to_string())?
        };

        // 2. 负 offset 自动算最后一页
        let mut real_offset = params.offset;
        if real_offset < 0 {
            real_offset = std::cmp::max(0, total_count - limit);
        }

        // 3. 数据查询
        let sql_data = format!(
            "SELECT id, timestamp, level, file, line, entity_id, entity_name, category, message \
             FROM logs {where_clause} ORDER BY id ASC LIMIT ? OFFSET ?"
        );
        let mut q = sqlx::query(&sql_data);
        for l in &levels_args {
            q = q.bind(l);
        }
        if let Some(eid) = entity_id_arg {
            q = q.bind(eid);
        }
        if let Some(ref cat) = category_arg {
            q = q.bind(cat);
        }
        if let Some(ref search) = search_arg {
            q = q.bind(search);
        }
        q = q.bind(limit);
        q = q.bind(real_offset);

        let rows = q.fetch_all(&pool).await.map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        for row in rows {
            result.push(LogRow {
                id: row.try_get(0).map_err(|e| e.to_string())?,
                timestamp: row.try_get(1).map_err(|e| e.to_string())?,
                level: row.try_get(2).map_err(|e| e.to_string())?,
                file: row.try_get(3).map_err(|e| e.to_string())?,
                line: row
                    .try_get::<Option<i64>, _>(4)
                    .map_err(|e| e.to_string())?
                    .map(|l| l as i32),
                entity_id: row
                    .try_get::<Option<i64>, _>(5)
                    .map_err(|e| e.to_string())?
                    .map(|e| e as i32),
                entity_name: row.try_get(6).map_err(|e| e.to_string())?,
                category: row.try_get(7).map_err(|e| e.to_string())?,
                message: row.try_get(8).map_err(|e| e.to_string())?,
            });
        }
        Ok(QueryLogsResult {
            rows: result,
            total_count: total_count as usize,
        })
    }

    async fn clear_logs(&self, _user_id: i32) -> Result<(), String> {
        let db_path = log_db_path()?;
        if !db_path.exists() {
            return Ok(());
        }
        use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
        let opts = SqliteConnectOptions::new().filename(&db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .map_err(|e| e.to_string())?;
        sqlx::query("DELETE FROM logs")
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
        pool.close().await;
        Ok(())
    }
}

// ── User Service Implementation ──

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    pub user_id: i32,
    pub exp: u64,
}

pub fn generate_jwt(user_id: i32) -> Result<String, String> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "moon-lol-secret-key-12345".to_string());
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() + 30 * 24 * 3600;

    let claims = JwtClaims { user_id, exp };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Token generation failed: {e}"))
}

pub struct UserServiceImpl {
    pub pool: PgPool,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn register(&self, phone: &str, password: &str, code: &str) -> Result<serde_json::Value, String> {
        if code != "111111" {
            return Err("验证码错误".to_string());
        }

        let exists = sqlx::query("SELECT 1 FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if exists.is_some() {
            return Err("该手机号已被注册".to_string());
        }

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| format!("密码加密失败: {e}"))?;

        let row = sqlx::query(
            "INSERT INTO users (phone, password_hash) VALUES ($1, $2) RETURNING id, phone"
        )
        .bind(phone)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("注册失败: {e}"))?;

        let id: i32 = row.try_get("id").map_err(|e| e.to_string())?;
        let phone: String = row.try_get("phone").map_err(|e| e.to_string())?;
        let token = generate_jwt(id)?;

        Ok(serde_json::json!({
            "token": token,
            "user": {
                "id": id,
                "phone": phone
            }
        }))
    }

    async fn login(&self, phone: &str, password: &str) -> Result<serde_json::Value, String> {
        let row = sqlx::query("SELECT id, phone, password_hash FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let Some(r) = row else {
            return Err("用户不存在或密码错误".to_string());
        };

        let password_hash: String = r.try_get("password_hash").map_err(|e| e.to_string())?;
        if !verify(password, &password_hash).map_err(|e| format!("密码校验失败: {e}"))? {
            return Err("用户不存在或密码错误".to_string());
        }

        let id: i32 = r.try_get("id").map_err(|e| e.to_string())?;
        let phone: String = r.try_get("phone").map_err(|e| e.to_string())?;
        let token = generate_jwt(id)?;

        Ok(serde_json::json!({
            "token": token,
            "user": {
                "id": id,
                "phone": phone
            }
        }))
    }

    async fn reset_password(&self, phone: &str, new_password: &str, code: &str) -> Result<(), String> {
        if code != "111111" {
            return Err("验证码错误".to_string());
        }

        let exists = sqlx::query("SELECT 1 FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if exists.is_none() {
            return Err("该手机号未注册".to_string());
        }

        let password_hash = hash(new_password, DEFAULT_COST)
            .map_err(|e| format!("密码加密失败: {e}"))?;

        sqlx::query("UPDATE users SET password_hash = $1 WHERE phone = $2")
            .bind(password_hash)
            .bind(phone)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("重置密码失败: {e}"))?;

        Ok(())
    }
}


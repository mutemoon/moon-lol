//! 领域类型 → 协议 DTO 的转换层。
//!
//! 所有 `From`/`Into` 实现均在本文件，服务器私有（不污染协议 crate）。
//! handler 层调用 service 拿到 domain 类型后，通过 `.into()` 转为协议 DTO 返回。

use lol_web_protocol as protocol;

// ── Team / Visibility（直接枚举映射） ──

impl From<crate::domain::spawn_preset::Team> for protocol::Team {
    fn from(t: crate::domain::spawn_preset::Team) -> Self {
        match t {
            crate::domain::spawn_preset::Team::Order => protocol::Team::Order,
            crate::domain::spawn_preset::Team::Chaos => protocol::Team::Chaos,
        }
    }
}

impl From<protocol::Team> for crate::domain::spawn_preset::Team {
    fn from(t: protocol::Team) -> Self {
        match t {
            protocol::Team::Order => crate::domain::spawn_preset::Team::Order,
            protocol::Team::Chaos => crate::domain::spawn_preset::Team::Chaos,
        }
    }
}

impl From<crate::domain::spawn_preset::Visibility> for protocol::Visibility {
    fn from(v: crate::domain::spawn_preset::Visibility) -> Self {
        match v {
            crate::domain::spawn_preset::Visibility::Private => protocol::Visibility::Private,
            crate::domain::spawn_preset::Visibility::Friends => protocol::Visibility::Friends,
            crate::domain::spawn_preset::Visibility::Public => protocol::Visibility::Public,
        }
    }
}

impl From<protocol::Visibility> for crate::domain::spawn_preset::Visibility {
    fn from(v: protocol::Visibility) -> Self {
        match v {
            protocol::Visibility::Private => crate::domain::spawn_preset::Visibility::Private,
            protocol::Visibility::Friends => crate::domain::spawn_preset::Visibility::Friends,
            protocol::Visibility::Public => crate::domain::spawn_preset::Visibility::Public,
        }
    }
}

// ── AgentType ──

impl From<crate::domain::agent::AgentType> for protocol::AgentType {
    fn from(t: crate::domain::agent::AgentType) -> Self {
        match t {
            crate::domain::agent::AgentType::Llm => protocol::AgentType::Llm,
            crate::domain::agent::AgentType::Rl => protocol::AgentType::Rl,
            crate::domain::agent::AgentType::Script => protocol::AgentType::Script,
        }
    }
}

impl From<protocol::AgentType> for crate::domain::agent::AgentType {
    fn from(t: protocol::AgentType) -> Self {
        match t {
            protocol::AgentType::Llm => crate::domain::agent::AgentType::Llm,
            protocol::AgentType::Rl => crate::domain::agent::AgentType::Rl,
            protocol::AgentType::Script => crate::domain::agent::AgentType::Script,
        }
    }
}

// ── Agent ──

impl From<crate::domain::agent::Agent> for protocol::agent::Agent {
    fn from(a: crate::domain::agent::Agent) -> Self {
        protocol::agent::Agent {
            id: a.id,
            owner_id: a.owner_id,
            name: a.name,
            champion: a.champion,
            agent_type: a.agent_type.into(),
            prompt: a.prompt,
            model: if a.model.is_empty() {
                None
            } else {
                Some(a.model)
            },
            config_json: if a.config_json.is_null() {
                None
            } else {
                Some(a.config_json)
            },
            visibility: a.visibility.into(),
            forked_from: a.forked_from,
            upstream_agent_id: a.upstream_agent_id,
            created_at: a.created_at.to_rfc3339(),
            updated_at: a.updated_at.to_rfc3339(),
        }
    }
}

impl From<protocol::agent::CreateAgentDto> for crate::domain::agent::AgentInput {
    fn from(dto: protocol::agent::CreateAgentDto) -> Self {
        crate::domain::agent::AgentInput {
            name: dto.name,
            champion: dto.champion,
            agent_type: dto.agent_type.into(),
            prompt: dto.prompt,
            model: dto.model.unwrap_or_default(),
            config_json: dto.config_json.unwrap_or(serde_json::Value::Null),
            visibility: dto
                .visibility
                .unwrap_or(protocol::Visibility::Private)
                .into(),
        }
    }
}

// ── AgentSnapshot ──

impl From<crate::domain::agent_snapshot::AgentSnapshot>
    for protocol::agent_snapshot::AgentSnapshot
{
    fn from(s: crate::domain::agent_snapshot::AgentSnapshot) -> Self {
        protocol::agent_snapshot::AgentSnapshot {
            id: s.id,
            agent_id: s.agent_id,
            version: s.version,
            config_freeze: s.config_freeze,
            created_at: s.published_at.to_rfc3339(),
        }
    }
}

// ── Scenario ──

impl From<crate::domain::scenario::Scenario> for protocol::scenario::Scenario {
    fn from(s: crate::domain::scenario::Scenario) -> Self {
        protocol::scenario::Scenario {
            id: s.id,
            owner_id: s.owner_id,
            name: s.name,
            agents: s.agents,
            win_condition: s.win_condition,
            created_at: Some(s.created_at.to_rfc3339()),
            updated_at: Some(s.updated_at.to_rfc3339()),
        }
    }
}

impl From<protocol::scenario::CreateScenarioDto> for crate::domain::scenario::ScenarioInput {
    fn from(dto: protocol::scenario::CreateScenarioDto) -> Self {
        crate::domain::scenario::ScenarioInput {
            name: dto.name,
            agents: dto.agents,
        }
    }
}

// ── SpawnPreset ──

impl From<crate::domain::spawn_preset::SpawnPreset> for protocol::spawn_preset::SpawnPreset {
    fn from(p: crate::domain::spawn_preset::SpawnPreset) -> Self {
        protocol::spawn_preset::SpawnPreset {
            id: p.id,
            owner_id: p.owner_id,
            name: p.name,
            x: p.x,
            z: p.z,
            team: p.team.into(),
            visibility: p.visibility.into(),
        }
    }
}

impl From<protocol::spawn_preset::CreateSpawnPresetDto>
    for crate::domain::spawn_preset::SpawnPresetInput
{
    fn from(dto: protocol::spawn_preset::CreateSpawnPresetDto) -> Self {
        crate::domain::spawn_preset::SpawnPresetInput {
            name: dto.name,
            x: dto.x,
            z: dto.z,
            team: dto.team.into(),
            visibility: dto.visibility.into(),
        }
    }
}

// ── Room ──

impl From<crate::domain::room::RoomConstraints> for protocol::room::RoomConstraints {
    fn from(c: crate::domain::room::RoomConstraints) -> Self {
        protocol::room::RoomConstraints {
            max_members: c.max_members,
            max_agents_per_member: c.max_agents_per_member,
            team_policy: match c.team_policy {
                crate::domain::room::TeamPolicy::SingleTeam => {
                    protocol::room::TeamPolicy::SingleTeam
                }
                crate::domain::room::TeamPolicy::Free => protocol::room::TeamPolicy::Free,
            },
            lobby_visible: c.lobby_visible,
            prompt_visible: c.prompt_visible,
        }
    }
}

impl From<protocol::room::RoomConstraints> for crate::domain::room::RoomConstraints {
    fn from(c: protocol::room::RoomConstraints) -> Self {
        crate::domain::room::RoomConstraints {
            max_members: c.max_members,
            max_agents_per_member: c.max_agents_per_member,
            team_policy: match c.team_policy {
                protocol::room::TeamPolicy::SingleTeam => {
                    crate::domain::room::TeamPolicy::SingleTeam
                }
                protocol::room::TeamPolicy::Free => crate::domain::room::TeamPolicy::Free,
            },
            lobby_visible: c.lobby_visible,
            prompt_visible: c.prompt_visible,
        }
    }
}

impl From<crate::domain::room::Room> for protocol::room::Room {
    fn from(r: crate::domain::room::Room) -> Self {
        protocol::room::Room {
            id: r.id,
            name: r.name,
            owner_id: r.owner_id,
            constraints: r.constraints.into(),
            invite_code: r.invite_code,
            member_count: r.member_count,
            status: match r.status {
                crate::domain::room::RoomStatus::Lobby => protocol::room::RoomStatus::Lobby,
                crate::domain::room::RoomStatus::Running => protocol::room::RoomStatus::Running,
                crate::domain::room::RoomStatus::Closed => protocol::room::RoomStatus::Closed,
            },
            created_at: Some(r.created_at.to_rfc3339()),
        }
    }
}

impl From<crate::domain::room::RoomAgentSlot> for protocol::room::RoomAgentSlot {
    fn from(s: crate::domain::room::RoomAgentSlot) -> Self {
        protocol::room::RoomAgentSlot {
            id: s.id,
            room_id: s.room_id,
            member_user_id: s.user_id,
            agent_id: s.agent_id,
            team: s.team.into(),
        }
    }
}

// ── Match ──

impl From<crate::domain::match_::MatchStatus> for protocol::MatchStatus {
    fn from(s: crate::domain::match_::MatchStatus) -> Self {
        match s {
            crate::domain::match_::MatchStatus::Pending => protocol::MatchStatus::Pending,
            crate::domain::match_::MatchStatus::Running => protocol::MatchStatus::Running,
            crate::domain::match_::MatchStatus::Paused => protocol::MatchStatus::Paused,
            crate::domain::match_::MatchStatus::Finished => protocol::MatchStatus::Finished,
            crate::domain::match_::MatchStatus::Aborted => protocol::MatchStatus::Aborted,
        }
    }
}

impl From<crate::domain::match_::Match> for protocol::match_::Match {
    fn from(m: crate::domain::match_::Match) -> Self {
        protocol::match_::Match {
            id: m.id,
            mode: m.mode,
            status: m.status.into(),
            owner_user_id: Some(m.owner_id),
            room_id: m.room_id,
            ws_port: m.bevy_port,
            created_at: m.created_at.to_rfc3339(),
            finished_at: m.finished_at.map(|t| t.to_rfc3339()),
        }
    }
}

impl From<crate::domain::match_::MatchEvent> for protocol::match_::MatchEvent {
    fn from(ev: crate::domain::match_::MatchEvent) -> Self {
        let mut payload = ev.payload.clone();
        if let serde_json::Value::Object(ref mut map) = payload {
            map.insert(
                "event_type".to_string(),
                serde_json::Value::String(ev.event_type),
            );
            if let Some(agent_id) = ev.agent_id {
                map.insert(
                    "agent_id".to_string(),
                    serde_json::Value::String(agent_id.to_string()),
                );
            }
            map.insert(
                "game_time_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from(ev.game_time_ms)),
            );
        }
        protocol::match_::MatchEvent {
            id: ev.id,
            match_id: ev.match_id,
            seq: ev.seq,
            payload,
            recorded_at: ev.occurred_at.to_rfc3339(),
        }
    }
}

// ── EssenceTransaction ──

impl From<crate::domain::essence::EssenceTransaction> for protocol::essence::EssenceTransaction {
    fn from(t: crate::domain::essence::EssenceTransaction) -> Self {
        protocol::essence::EssenceTransaction {
            id: t.id,
            user_id: t.user_id,
            amount: t.delta,
            reason: t.reason,
            created_at: t.created_at.to_rfc3339(),
        }
    }
}

// ── BillingPlan ──

impl From<crate::domain::essence::BillingPlan> for protocol::essence::BillingPlan {
    fn from(p: crate::domain::essence::BillingPlan) -> Self {
        protocol::essence::BillingPlan {
            id: p.id,
            name: p.name,
            monthly_essence: p.essence_per_month,
            agent_limit: p.max_agents,
            price_cents: p.price_cents,
        }
    }
}

// ── ModelProvider (Dto) ──

impl From<crate::domain::model_provider::ModelProviderDto>
    for protocol::model_provider::ModelProvider
{
    fn from(dto: crate::domain::model_provider::ModelProviderDto) -> Self {
        protocol::model_provider::ModelProvider {
            id: dto.id,
            name: dto.name,
            category: dto.category,
            preset_type: dto.preset_type,
            base_url: dto.base_url,
            api_key: dto.api_key,
            has_api_key: dto.has_api_key,
            api_format: dto.api_format,
            models: dto
                .models
                .into_iter()
                .map(|m| protocol::model_provider::ModelConfig {
                    name: m.name,
                    max_tokens: m.max_tokens,
                })
                .collect(),
            enabled: dto.enabled,
            website_url: if dto.website_url.is_empty() {
                None
            } else {
                Some(dto.website_url)
            },
            api_key_url: if dto.api_key_url.is_empty() {
                None
            } else {
                Some(dto.api_key_url)
            },
            icon: if dto.icon.is_empty() {
                None
            } else {
                Some(dto.icon)
            },
            icon_color: if dto.icon_color.is_empty() {
                None
            } else {
                Some(dto.icon_color)
            },
            sort_order: dto.sort_order,
        }
    }
}

impl From<protocol::model_provider::ModelProviderInput>
    for crate::domain::model_provider::ModelProviderInput
{
    fn from(input: protocol::model_provider::ModelProviderInput) -> Self {
        crate::domain::model_provider::ModelProviderInput {
            name: input.name,
            category: input.category,
            preset_type: input.preset_type,
            base_url: input.base_url,
            api_key: input.api_key,
            api_format: input.api_format,
            models: input
                .models
                .into_iter()
                .map(|m| lol_agent_runtime::ModelConfig {
                    name: m.name,
                    max_tokens: m.max_tokens,
                })
                .collect(),
            enabled: input.enabled,
            website_url: input.website_url,
            api_key_url: input.api_key_url,
            icon: input.icon,
            icon_color: input.icon_color,
            sort_order: input.sort_order,
        }
    }
}

// ── Rank ──

impl From<crate::repository::rank_repo::RankQueueEntry> for protocol::rank::RankQueueEntry {
    fn from(e: crate::repository::rank_repo::RankQueueEntry) -> Self {
        protocol::rank::RankQueueEntry {
            user_id: e.user_id,
            agent_id: e.agent_id,
            agent_snapshot_id: e.agent_snapshot_id,
            mode: e.mode,
            rating: e.rating,
            enqueued_at: e.enqueued_at.to_rfc3339(),
        }
    }
}

impl From<crate::repository::rank_repo::EloRating> for protocol::rank::EloRating {
    fn from(r: crate::repository::rank_repo::EloRating) -> Self {
        protocol::rank::EloRating {
            agent_id: r.agent_id,
            agent_name: r.agent_name,
            mode: r.mode,
            rating: r.rating,
            games_played: r.wins + r.losses + r.draws,
            wins: r.wins,
            losses: r.losses,
            daily_delta: r.daily_delta,
        }
    }
}

impl From<crate::repository::rank_repo::Season> for protocol::rank::Season {
    fn from(s: crate::repository::rank_repo::Season) -> Self {
        protocol::rank::Season {
            id: s.id,
            mode: s.mode,
            starts_at: s.starts_at.to_rfc3339(),
            ends_at: Some(s.ends_at.to_rfc3339()),
        }
    }
}

// ── AdminMetrics ──

impl From<crate::service::admin_service::AdminMetrics> for protocol::admin::AdminMetrics {
    fn from(m: crate::service::admin_service::AdminMetrics) -> Self {
        protocol::admin::AdminMetrics {
            running_matches: m.running_matches,
            pending_matches: m.pending_matches,
            queued_agents: m.queued_agents,
            managed_processes: m.managed_processes,
        }
    }
}

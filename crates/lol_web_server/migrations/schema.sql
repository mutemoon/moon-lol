-- MoonLOL 全量数据库 Schema
-- 对应 docs/API_DESIGN.md §2 数据模型
-- 所有子系统共用；testcontainers fixture 与生产 init_db 均执行本文件。

-- ── 用户与配置 ──
CREATE TABLE IF NOT EXISTS users (
    id            SERIAL PRIMARY KEY,
    phone         VARCHAR(20) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS ai_config (
    user_id  INT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    api_key  TEXT NOT NULL DEFAULT '',
    base_url TEXT NOT NULL DEFAULT '',
    preamble TEXT NOT NULL DEFAULT ''
);

-- ── Agent 资产三件套 ──
CREATE TABLE IF NOT EXISTS spawn_presets (
    id         UUID PRIMARY KEY,
    owner_id   INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    x          REAL NOT NULL,
    z          REAL NOT NULL,
    team       TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'private',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);

CREATE TABLE IF NOT EXISTS agents (
    id                UUID PRIMARY KEY,
    owner_id          INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name              TEXT NOT NULL,
    champion          TEXT NOT NULL,
    agent_type        TEXT NOT NULL,
    prompt            TEXT NOT NULL DEFAULT '',
    preamble          TEXT NOT NULL DEFAULT '',
    model             TEXT NOT NULL DEFAULT '',
    config_json       JSONB NOT NULL DEFAULT '{}',
    visibility        TEXT NOT NULL DEFAULT 'private',
    forked_from       UUID NULL REFERENCES agents(id) ON DELETE SET NULL,
    upstream_agent_id UUID NULL REFERENCES agents(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);

CREATE TABLE IF NOT EXISTS agent_snapshots (
    id            UUID PRIMARY KEY,
    agent_id      UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    version       INT NOT NULL,
    config_freeze JSONB NOT NULL,
    published_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (agent_id, version)
);

-- ── 场景 ──
CREATE TABLE IF NOT EXISTS scenarios (
    id         UUID PRIMARY KEY,
    owner_id   INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    agents     JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name)
);

CREATE TABLE IF NOT EXISTS scenario_win_conditions (
    owner_id    INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scenario_id UUID NOT NULL REFERENCES scenarios(id) ON DELETE CASCADE,
    condition   JSONB NOT NULL,
    PRIMARY KEY (owner_id, scenario_id)
);

-- ── 房间子系统 ──
CREATE TABLE IF NOT EXISTS rooms (
    id                     UUID PRIMARY KEY,
    owner_id               INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                   TEXT NOT NULL,
    invite_code            TEXT UNIQUE NOT NULL,
    max_members            INT NOT NULL DEFAULT 10,
    max_agents_per_member  INT NOT NULL DEFAULT 3,
    team_policy            TEXT NOT NULL DEFAULT 'free',
    lobby_visible          BOOLEAN NOT NULL DEFAULT TRUE,
    prompt_visible         BOOLEAN NOT NULL DEFAULT FALSE,
    status                 TEXT NOT NULL DEFAULT 'lobby',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS room_members (
    room_id   UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id   INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_ready  BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (room_id, user_id)
);

CREATE TABLE IF NOT EXISTS room_agent_slots (
    id              UUID PRIMARY KEY,
    room_id         UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    team            TEXT NOT NULL,
    spawn_preset_id UUID NULL REFERENCES spawn_presets(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ── 对局实例（核心表）──
CREATE TABLE IF NOT EXISTS matches (
    id             UUID PRIMARY KEY,
    form           TEXT NOT NULL,
    room_id        UUID NULL REFERENCES rooms(id) ON DELETE SET NULL,
    rank_queue_id  UUID NULL,
    owner_id       INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode           TEXT NOT NULL,
    scenario_id    UUID NULL REFERENCES scenarios(id) ON DELETE SET NULL,
    win_condition  JSONB NULL,
    status         TEXT NOT NULL DEFAULT 'pending',
    bevy_port      INT NULL,
    ws_port        INT NULL,
    started_at     TIMESTAMPTZ NULL,
    finished_at    TIMESTAMPTZ NULL,
    winner_team    TEXT NULL,
    abort_reason   TEXT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_matches_owner ON matches(owner_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_matches_room ON matches(room_id) WHERE room_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(status);

CREATE TABLE IF NOT EXISTS match_participants (
    id                UUID PRIMARY KEY,
    match_id          UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    agent_snapshot_id UUID NOT NULL REFERENCES agent_snapshots(id) ON DELETE RESTRICT,
    agent_id          UUID NOT NULL REFERENCES agents(id) ON DELETE RESTRICT,
    user_id           INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    team              TEXT NOT NULL,
    bevy_entity_id    BIGINT NULL,
    result            TEXT NULL,
    final_stats       JSONB NULL,
    UNIQUE (match_id, agent_snapshot_id)
);
CREATE INDEX IF NOT EXISTS idx_match_part_match ON match_participants(match_id);
CREATE INDEX IF NOT EXISTS idx_match_part_agent ON match_participants(agent_id);

CREATE TABLE IF NOT EXISTS match_events (
    id           BIGSERIAL PRIMARY KEY,
    match_id     UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    seq          INT NOT NULL,
    event_type   TEXT NOT NULL,
    agent_id     UUID NULL,
    payload      JSONB NOT NULL,
    game_time_ms BIGINT NOT NULL,
    occurred_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (match_id, seq)
);
CREATE INDEX IF NOT EXISTS idx_match_events_match ON match_events(match_id, seq);

-- ── Rank 子系统 ──
CREATE TABLE IF NOT EXISTS seasons (
    id         UUID PRIMARY KEY,
    name       TEXT NOT NULL,
    mode       TEXT NOT NULL,
    starts_at  TIMESTAMPTZ NOT NULL,
    ends_at    TIMESTAMPTZ NOT NULL,
    status     TEXT NOT NULL DEFAULT 'scheduled',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_seasons_mode_status ON seasons(mode, status);

CREATE TABLE IF NOT EXISTS rank_queues (
    id                UUID PRIMARY KEY,
    agent_id          UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    agent_snapshot_id UUID NOT NULL REFERENCES agent_snapshots(id) ON DELETE CASCADE,
    user_id           INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode              TEXT NOT NULL,
    season_id         UUID NOT NULL REFERENCES seasons(id) ON DELETE RESTRICT,
    status            TEXT NOT NULL DEFAULT 'queued',
    enqueued_at       TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_match_at     TIMESTAMPTZ NULL,
    UNIQUE (agent_snapshot_id, season_id)
);
CREATE INDEX IF NOT EXISTS idx_rank_queue_status ON rank_queues(mode, status);

CREATE TABLE IF NOT EXISTS elo_ratings (
    id         UUID PRIMARY KEY,
    agent_id   UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    mode       TEXT NOT NULL,
    season_id  UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    rating     DOUBLE PRECISION NOT NULL DEFAULT 1200,
    wins       INT NOT NULL DEFAULT 0,
    losses     INT NOT NULL DEFAULT 0,
    draws      INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (agent_id, mode, season_id)
);
CREATE INDEX IF NOT EXISTS idx_elo_leaderboard ON elo_ratings(mode, season_id, rating DESC);

-- ── 精粹 / 订阅 ──
CREATE TABLE IF NOT EXISTS essence_balances (
    user_id    INT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    amount     BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS essence_transactions (
    id            BIGSERIAL PRIMARY KEY,
    user_id       INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    delta         BIGINT NOT NULL,
    reason        TEXT NOT NULL,
    reference     TEXT NULL,
    balance_after BIGINT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_essence_tx_user ON essence_transactions(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS billing_plans (
    id                TEXT PRIMARY KEY,
    name              TEXT NOT NULL,
    price_cents       INT NOT NULL,
    essence_per_month BIGINT NOT NULL,
    max_agents        INT NOT NULL,
    features_json     JSONB NOT NULL DEFAULT '{}',
    sort_order        INT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS subscriptions (
    id           UUID PRIMARY KEY,
    user_id      INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    plan_id      TEXT NOT NULL REFERENCES billing_plans(id),
    status       TEXT NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end   TIMESTAMPTZ NOT NULL,
    auto_renew   BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON subscriptions(user_id, status);

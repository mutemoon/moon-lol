CREATE TABLE IF NOT EXISTS rl_tasks (
    id           UUID PRIMARY KEY,
    name         TEXT NOT NULL,
    agent_type   TEXT NOT NULL DEFAULT 'PPO (Candle)',
    env_name     TEXT NOT NULL DEFAULT 'FioraV0',
    status       TEXT NOT NULL DEFAULT 'queued',
    config_json  JSONB NOT NULL DEFAULT '{}',
    current_step BIGINT NOT NULL DEFAULT 0,
    ep_return    REAL NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rl_checkpoints (
    id         UUID PRIMARY KEY,
    task_id    UUID NOT NULL REFERENCES rl_tasks(id) ON DELETE CASCADE,
    step       BIGINT NOT NULL,
    path       TEXT NOT NULL,
    ep_return  REAL NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_rl_checkpoints_task_id ON rl_checkpoints(task_id);

CREATE TABLE IF NOT EXISTS rl_metrics (
    id          BIGSERIAL PRIMARY KEY,
    task_id     UUID NOT NULL REFERENCES rl_tasks(id) ON DELETE CASCADE,
    step        BIGINT NOT NULL,
    ep_return   REAL NOT NULL,
    loss        REAL NOT NULL,
    policy_loss REAL NOT NULL DEFAULT 0,
    value_loss  REAL NOT NULL DEFAULT 0,
    total_loss  REAL NOT NULL DEFAULT 0,
    kl          REAL NOT NULL,
    entropy     REAL NOT NULL,
    clip_frac   REAL NOT NULL DEFAULT 0,
    value       REAL NOT NULL,
    fps         INTEGER NOT NULL,
    ep_steps_max BIGINT NOT NULL DEFAULT 0,
    ep_steps_min BIGINT NOT NULL DEFAULT 0,
    ep_steps_avg REAL NOT NULL DEFAULT 0,
    reward_breakdown JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_rl_metrics_task_id ON rl_metrics(task_id, step);

CREATE TABLE IF NOT EXISTS rl_logs (
    id          BIGSERIAL PRIMARY KEY,
    task_id     UUID NOT NULL REFERENCES rl_tasks(id) ON DELETE CASCADE,
    level       TEXT NOT NULL,
    message     TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_rl_logs_task_id ON rl_logs(task_id);

ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS ep_steps_max BIGINT NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS ep_steps_min BIGINT NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS ep_steps_avg REAL NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS policy_loss REAL NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS value_loss REAL NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS total_loss REAL NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS clip_frac REAL NOT NULL DEFAULT 0;
ALTER TABLE rl_metrics ADD COLUMN IF NOT EXISTS reward_breakdown JSONB NOT NULL DEFAULT '[]'::jsonb;


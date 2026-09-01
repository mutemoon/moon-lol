// =============================================================================
// FioraV3 规范：剑姬补刀训练强化学习环境 (Solo 单人补刀地图)
// =============================================================================

env FioraV3 {
    label: "剑姬补刀训练 (Solo地图-补刀效率)"
    tag: "FioraV3"
    description: "剑姬在召唤师峡谷上路Solo地图进行单人补刀训练（补刀成功奖励，普通攻击未补刀惩罚）"
    num_agents: 1
    params {
        lr: 0.0003
        gamma: 0.99
        gae_lambda: 0.95
        clip_eps: 0.2
        ppo_epochs: 4
        hidden_dim: 64
        rollout_steps_per_env: 160
        total_iterations: 300
    }
}

// ── 观测空间 ─────────────────────────────────────────────────────────────────
obs FioraV3Obs {
    struct attack {
        scalar is_ready = attack_is_ready;
        scalar is_windup = attack_is_windup;
        scalar is_cooldown = attack_is_cooldown;
        scalar timer_remaining = attack_timer_remaining;
        scalar self_ad_norm = self_ad / 100.0;
    }

    repeated visible_units[12] -> encoder: SharedMlpPool(hidden=[64, 32], pool=Max) {
        category unit_type: 6 -> embed(8) = unit_type;
        vector rel_pos: 2 = [rel_pos[0] / 1000.0, rel_pos[1] / 1000.0];
        scalar hp_pct = hp_pct;
        scalar hp_norm = hp_norm;
        scalar is_enemy = is_enemy;
    }

    repeated visible_missiles[4] -> encoder: SharedMlpPool(hidden=[16, 8], pool=Max) {
        vector rel_pos: 2 = [rel_pos[0] / 1000.0, rel_pos[1] / 1000.0];
        scalar is_enemy = is_enemy;
        scalar is_active = is_active;
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────
action FioraV3Action {
    continuous offset: 2;
    category action_type: 3 {
        0: "保持当前 (NoOp)",
        1: "移动 (Move)",
        2: "普通攻击 (Attack)",
    }
    unit_target target: visible_units[12 -> 32];

    mask {
        // ① 目标实体槽位有效性过滤：当槽位单位无效 (unit_type <= 0) 时禁用该 target 槽位
        for u in visible_units {
            if u.unit_type <= 0.0 { disable target; }
        }

        // ② 全局基础冷却过滤
        if attack_is_cooldown > 0.5 { disable Attack; }

        // ③ 针对选中动作的条件目标过滤：普通攻击禁止以友军或非敌军为目标
        for u in visible_units {
            if u.is_enemy <= 0.5 {
                disable Attack;
            }
        }
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward FioraV3Reward {
    term last_hit             : "补刀成功奖励"     = cs_reward_coef * self_cs;
    term attack_no_cs_penalty : "攻击小兵未补刀惩罚" = -0.01 * (penalty_coef * self_attack_no_cs);
}
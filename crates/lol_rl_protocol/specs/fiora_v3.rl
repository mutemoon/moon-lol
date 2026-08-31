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
    category role: 4 -> embed(12) = role_id;

    struct attack {
        scalar is_ready = attack_is_ready;
        scalar is_windup = attack_is_windup;
        scalar is_cooldown = attack_is_cooldown;
        scalar timer_remaining = attack_timer_remaining;
        scalar self_ad_norm = self_ad / 100.0;
    }

    struct health {
        scalar self_hp_pct = clamp(self_hp / max(self_max_hp, 1.0), 0.0, 1.0);
        scalar self_hp_norm = self_hp / 1000.0;
    }

    repeated self_modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 11 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }

    repeated visible_units[20] -> encoder: SharedMlpPool(hidden=[64, 32], pool=Max) {
        category unit_type: 6 -> embed(8) = unit_type;
        vector rel_pos: 2 = [rel_pos[0] / 100.0, rel_pos[1] / 100.0];
        scalar hp_pct = hp_pct;
        scalar hp_norm = hp_norm;
        scalar is_enemy = is_enemy;
    }

    repeated visible_missiles[4] -> encoder: SharedMlpPool(hidden=[16, 8], pool=Max) {
        vector rel_pos: 2 = [rel_pos[0] / 100.0, rel_pos[1] / 100.0];
        scalar is_enemy = is_enemy;
        scalar is_active = is_active;
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────
action FioraV3Action {
    continuous offset: 2;
    unit_target target: visible_units[20 -> 32];
    category action_type: 3 {
        0: "保持当前 (NoOp)",
        1: "移动 (Move)",
        2: "普通攻击 (Attack)",
    }

    mask {
        // ① 目标实体槽位有效性过滤：当槽位单位无效 (unit_type <= 0) 时禁用该 target 槽位
        for u in visible_units {
            if u.unit_type <= 0.0 { disable target; }
        }

        // ② 全局基础冷却过滤
        if attack_is_cooldown > 0.5 { disable Attack; }

        // ③ 针对选中目标的条件动作过滤：友军或非敌军目标禁止普通攻击
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
    term attack_no_cs_penalty : "攻击小兵未补刀惩罚" = -1.0 * (penalty_coef * self_attack_no_cs);
}
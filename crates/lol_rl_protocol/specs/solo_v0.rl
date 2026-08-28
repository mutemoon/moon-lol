// =============================================================================
// SoloV0 规范：剑姬 vs 瑞雯 1v1 对战强化学习环境
// =============================================================================

env SoloV0 {
    label: "剑姬 vs 瑞雯 (Solo 1v1 自博弈)"
    tag: "SoloV0"
    description: "单神经网络通过 role_id (0:剑姬, 1:瑞雯) 自博弈对抗，对称零和奖励与自我中心化全技能对决"
    num_agents: 2
    params {
        lr: 0.0003
        gamma: 0.99
        gae_lambda: 0.95
        clip_eps: 0.2
        ppo_epochs: 8
        hidden_dim: 64
        rollout_steps_per_env: 160
        total_iterations: 500
    }
}

// ── 观测空间 ─────────────────────────────────────────────────────────────────
obs SoloV0Obs {
    category role: 4 -> embed(12) = role_id;

    struct spatial {
        vector target_rel_pos: 2 = [
            (self_x - target_x) / 100.0,
            (self_z - target_z) / 100.0
        ];
        scalar distance = distance / 100.0;
    }

    struct attack {
        scalar is_ready = attack_is_ready;
        scalar is_windup = attack_is_windup;
        scalar is_cooldown = attack_is_cooldown;
        scalar timer_remaining = attack_timer_remaining;
    }

    struct cooldowns {
        scalar q_ready = q_ready;
        scalar q_cd = q_cd / 10.0;
        scalar w_ready = w_ready;
        scalar w_cd = w_cd / 15.0;
        scalar e_ready = e_ready;
        scalar e_cd = e_cd / 10.0;
        scalar r_ready = r_ready;
        scalar r_cd = r_cd / 60.0;
        scalar flash_ready = flash_ready;
        scalar flash_cd = flash_cd / 300.0;
    }

    struct health {
        scalar self_hp_pct = clamp(self_hp / max(self_max_hp, 1.0), 0.0, 1.0);
        scalar target_hp_pct = clamp(target_hp / max(target_max_hp, 1.0), 0.0, 1.0);
    }

    repeated self_modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 11 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }

    repeated target_modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 11 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }

    repeated visible_units[20] -> encoder: SharedMlpPool(hidden=[32, 16], pool=Max) {
        category unit_type: 6 -> embed(8) = unit_type;
        vector rel_pos: 2 = [rel_pos[0] / 100.0, rel_pos[1] / 100.0];
        scalar hp_pct = hp_pct;
        scalar is_enemy = is_enemy;
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────
action SoloV0Action {
    continuous offset: 2;
    unit_target target: visible_units[20 -> 16];
    category action_type: 8 {
        0: "保持当前 (NoOp)",
        1: "移动 (Move)",
        2: "普通攻击 (Attack)",
        3: "施放 Q",
        4: "施放 W",
        5: "施放 E",
        6: "施放 R",
        7: "闪现",
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward SoloV0Reward {
    term last_hit             : "补刀成功奖励"     = cs_reward_coef * self_cs;
    term attack_no_cs_penalty : "攻击小兵未补刀惩罚" = -1.0 * (penalty_coef * self_attack_no_cs);
    term harass               : "消耗对手奖励"     = harass_coef * (self_harass_dmg - target_harass_dmg);
}

// =============================================================================
// FioraV2 规范：剑姬 vs 瑞雯 对战强化学习环境（全技能实战）
// =============================================================================

env FioraV2 {
    label: "剑姬 vs 瑞雯 (全技能实战-10f)"
    tag: "FioraV2"
    description: "剑姬 vs 瑞雯 对战强化学习环境（V2：全技能Q/E/R+闪现+普攻+NoOp，10帧物理推演）"
    num_agents: 1
    params {
        lr: 0.0003
        gamma: 0.99
        gae_lambda: 0.95
        clip_eps: 0.2
        ppo_epochs: 8
        hidden_dim: 64
        rollout_steps_per_env: 128
        total_iterations: 300
    }
}

// ── 观测空间 ─────────────────────────────────────────────────────────────────
obs FioraV2Obs {
    struct vital {
        vector direction: 4 = [vital_dir_x, vital_dir_neg_x, vital_dir_z, vital_dir_neg_z];
        scalar has_vital = has_vital;
        scalar is_active = vital_is_active;
    }

    struct spatial {
        vector relative_pos: 2 = [
            (fiora_x - riven_x) / 500.0,
            (fiora_z - riven_z) / 500.0
        ];
        scalar distance = distance / 500.0;
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
        scalar e_ready = e_ready;
        scalar e_cd = e_cd / 10.0;
        scalar r_ready = r_ready;
        scalar r_cd = r_cd / 60.0;
        scalar flash_ready = flash_ready;
        scalar flash_cd = flash_cd / 300.0;
    }

    struct health {
        scalar riven_hp_pct = clamp(riven_hp / max(riven_max_hp, 1.0), 0.0, 1.0);
    }

    repeated modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 11 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────
action FioraV2Action {
    continuous offset: 2;
    category action_type: 7 {
        0: "保持当前 (NoOp)",
        1: "移动 (Move)",
        2: "普通攻击 (Attack)",
        3: "Q-破空斩 (CastQ)",
        4: "E-夺命连刺 (CastE)",
        5: "R-无双挑战 (CastR)",
        6: "闪现 (CastFlash)",
    }

    mask {
        if distance > 22.0 { disable 2; }
        if q_ready < 0.5   { disable 3; }
        if e_ready < 0.5   { disable 4; }
        if r_ready < 0.5   { disable 5; }
        if flash_ready < 0.5 { disable 6; }
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward FioraV2Reward {
    term step_cost   : "时间惩罚" = -0.005;
    term vital_hit   : "击破破绽" = 80.0 * is_vital_break;
    term align       : "破绽对齐" = 5.0 * is_newly_aligned;
    term distance    : "接近奖励" = -0.01 * (distance / 500.0);
    term damage_deal : "造成伤害" = 100.0 * (damage_dealt / 10000.0);
}

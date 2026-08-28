// =============================================================================
// FioraV1 规范：剑姬 vs 瑞雯 对战强化学习环境（真实物理移动）
// =============================================================================

env FioraV1 {
    label: "剑姬 vs 瑞雯 (真实物理移动-6动作)"
    tag: "FioraV1"
    description: "剑姬 vs 瑞雯 对战强化学习环境（V1：真实物理移动 + 普攻，混合动作空间）"
    num_agents: 1
    params {
        lr: 0.0003
        gamma: 0.99
        gae_lambda: 0.95
        clip_eps: 0.2
        ppo_epochs: 4
        hidden_dim: 64
        rollout_steps_per_env: 128
        total_iterations: 200
    }
}

// ── 观测空间 ─────────────────────────────────────────────────────────────────
obs FioraV1Obs {
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
}

// ── 动作空间 ─────────────────────────────────────────────────────────────────
action FioraV1Action {
    continuous move: 2;
    category action_type: 2 {
        0: "移动 (Move)",
        1: "普通攻击 (Attack)",
    }

    mask {
        if distance > 22.0 { disable 1; }
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward FioraV1Reward {
    term time_penalty : "时间惩罚" = -0.002;
    term align        : "破绽对齐" = 0.02 * is_newly_aligned;
    term vital_hit    : "击破破绽" = 0.8 * is_vital_break;
}

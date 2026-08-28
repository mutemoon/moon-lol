// =============================================================================
// FioraV0 规范：剑姬 vs 瑞雯 对战强化学习环境（瞬移基准）
// =============================================================================

env FioraV0 {
    label: "剑姬 vs 瑞雯 (瞬移-5动作)"
    tag: "FioraV0"
    description: "剑姬 vs 瑞雯 对战强化学习环境（V0 基准：瞬移走位 + 普攻，5 离散动作）"
    num_agents: 1
    params {
        lr: 0.0003
        gamma: 0.99
        gae_lambda: 0.95
        clip_eps: 0.2
        ppo_epochs: 4
        hidden_dim: 64
        rollout_steps_per_env: 128
        total_iterations: 100
    }
}

// ── 观测空间 ─────────────────────────────────────────────────────────────────
obs FioraV0Obs {
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
action FioraV0Action {
    category action_type: 5 {
        0: "东移 50u",
        1: "西移 50u",
        2: "北移 50u",
        3: "南移 50u",
        4: "攻击瑞雯",
    }

    mask {
        if distance > 22.0 { disable 4; }
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward FioraV0Reward {
    term time_penalty     : "时间惩罚 (Time Penalty)"               = -0.002;
    term alignment        : "对齐破绽方向 (Alignment Bonus)"         = 0.02 * is_newly_aligned;
    term misalignment     : "错误方向移动 (Misalignment Penalty)"     = -0.02 * is_misaligned_move;
    term attack_miss      : "空挥攻击 (Attack Miss Penalty)"         = -0.1 * is_attack_missed;
    term vital_break      : "打破绽成功 (Vital Break)"               = 0.8 * is_vital_break;
    term kill_reward      : "击杀基础奖励 (Kill Reward)"             = 2.0 * is_kill;
    term quick_kill_bonus : "极速击杀时效奖励 (Quick Kill Time Reward)" = if(is_kill > 0.5, quick_kill_reward, 0.0);
}

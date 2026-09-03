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

    struct skills {
        scalar level = hero_level;
        scalar skill_points = skill_points;
        scalar q_level = q_level;
        scalar w_level = w_level;
        scalar e_level = e_level;
        scalar r_level = r_level;
        scalar q_ready = q_ready;
        scalar w_ready = w_ready;
        scalar e_ready = e_ready;
        scalar r_ready = r_ready;
        scalar can_cast_any = can_cast_any;
        scalar can_level_up_any = can_level_up_any;
        scalar can_level_up_q = can_level_up_q;
        scalar can_level_up_w = can_level_up_w;
        scalar can_level_up_e = can_level_up_e;
        scalar can_level_up_r = can_level_up_r;
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
    category action_type: 5 {
        0: "保持当前 (NoOp)",
        1: "移动 (Move)",
        2: "普通攻击 (Attack)",
        3: "施放技能 (CastSkill)",
        4: "升级技能 (LevelUpSkill)",
    }
    category skill_slot: 4 {
        0: "Q",
        1: "W",
        2: "E",
        3: "R",
    }
    unit_target target: visible_units[12 -> 32];

    mask {
        // ① 目标实体槽位有效性过滤：当槽位单位无效 (unit_type <= 0) 时禁用该 target 槽位
        for u in visible_units {
            if u.unit_type <= 0.0 { disable target; }
        }

        // ② 全局基础冷却与可用性过滤
        if attack_is_cooldown > 0.5 { disable Attack; }
        if can_cast_any < 0.5       { disable CastSkill; }
        if can_level_up_any < 0.5   { disable LevelUpSkill; }

        // ③ 当主动作选择 CastSkill 时，对技能槽位施加施放冷却与学习状态过滤，且禁止以友军为目标
        when CastSkill {
            if q_ready < 0.5 { disable skill_slot.Q; }
            if w_ready < 0.5 { disable skill_slot.W; }
            if e_ready < 0.5 { disable skill_slot.E; }
            if r_ready < 0.5 { disable skill_slot.R; }

            for u in visible_units {
                if u.is_enemy <= 0.5 {
                    disable target;
                }
            }
        }

        // ④ 当主动作选择 LevelUpSkill 时，对技能槽位施加升级规则过滤
        when LevelUpSkill {
            if can_level_up_q < 0.5 { disable skill_slot.Q; }
            if can_level_up_w < 0.5 { disable skill_slot.W; }
            if can_level_up_e < 0.5 { disable skill_slot.E; }
            if can_level_up_r < 0.5 { disable skill_slot.R; }
        }

        // ⑤ 针对普通攻击的条件目标过滤：普通攻击禁止以友军或非敌军为目标
        when Attack {
            for u in visible_units {
                if u.is_enemy <= 0.5 {
                    disable target;
                }
            }
        }
    }
}

// ── 奖励公式 ─────────────────────────────────────────────────────────────────
reward FioraV3Reward {
    term last_hit             : "补刀成功奖励"     = cs_reward_coef * self_cs;
    term attack_no_cs_penalty : "攻击小兵未补刀惩罚" = -0.1 * (penalty_coef * self_attack_no_cs);
}
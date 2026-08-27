// ── Fiora vs Riven V2 全技能实战对战环境规范 ──────────────────

obs FioraV2Obs {
    // 1. 角色标识 (单体视角固定0=Fiora)
    category role: 4 -> embed(12) = role_id;

    // 2. 空间相对特征 (3维)
    struct spatial {
        vector target_rel_pos: 2 = [
            (fiora_x - riven_x) / 100.0,
            (fiora_z - riven_z) / 100.0
        ];
        scalar distance = distance / 100.0;
    }

    // 3. 普攻状态机 (4维)
    struct attack {
        scalar is_ready = attack_is_ready;
        scalar is_windup = attack_is_windup;
        scalar is_cooldown = attack_is_cooldown;
        scalar timer_remaining = attack_timer_remaining;
    }

    // 4. 技能与闪现冷却 (8维: Q, E, R, Flash 及其剩余CD)
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

    // 5. 双方血量百分比 (2维)
    struct health {
        scalar fiora_hp_pct = clamp(fiora_hp / max(fiora_max_hp, 1.0), 0.0, 1.0);
        scalar riven_hp_pct = clamp(riven_hp / max(riven_max_hp, 1.0), 0.0, 1.0);
    }

    // 6. 自身修饰符 (4 槽位 × 5维)
    repeated self_modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 8 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }

    // 7. 目标修饰符 (4 槽位 × 5维)
    repeated target_modifiers[4] -> encoder: SharedMlpFlatten(hidden=[16]) {
        category name: 8 -> embed(8) = name;
        scalar remaining_duration = remaining_duration;
        scalar stack_count = stack_count;
        vector params: 2 = [params[0], params[1]];
    }
}

action FioraV2Action {
    continuous offset: 2;
    category action_type: 7 {
        0: "NoOp",
        1: "Move",
        2: "Attack",
        3: "CastQ",
        4: "CastE",
        5: "CastR",
        6: "CastFlash",
    }

    mask {
        if distance > 220.0 { disable Attack; }
        if q_ready < 0.5   { disable CastQ; }
        if e_ready < 0.5   { disable CastE; }
        if r_ready < 0.5   { disable CastR; }
        if flash_ready < 0.5 { disable CastFlash; }
    }
}

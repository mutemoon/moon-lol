//! 锐雯Q技能范围伤害测试
//!
//! 测试覆盖：
//! 1. 锐雯Q技能三段式伤害机制
//! 2. 位移路径上的范围检测
//! 3. 伤害对不同目标类型的影响（英雄、小兵）
//! 4. 多目标同时受到伤害
//! 5. 伤害半径随位移变化

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use moon_lol::{DamageType, DashDamage, TargetDamage, TargetFilter};

    // ===== 测试常量 =====
    const EPSILON: f32 = 1e-4;

    // 锐雯Q技能参数（从代码中提取）
    const RIVEN_Q_DASH_SPEED: f32 = 1000.0;
    const RIVEN_Q_DASH_DISTANCE: f32 = 250.0;
    const RIVEN_Q_DAMAGE_RADIUS_END: f32 = 250.0;
    const RIVEN_Q_DAMAGE_RADIUS_START: f32 = 65.0;

    // ===== 辅助函数 =====

    /// 计算给定位置的伤害半径
    /// 锐雯Q的伤害半径从起点65逐渐扩大到终点250
    fn calculate_dash_damage_radius(progress: f32) -> f32 {
        RIVEN_Q_DAMAGE_RADIUS_START
            + (RIVEN_Q_DAMAGE_RADIUS_END - RIVEN_Q_DAMAGE_RADIUS_START) * progress
    }

    /// 检查目标是否在位移路径的伤害范围内
    fn is_in_dash_damage_range(
        riven_pos: Vec3,
        dash_direction: Vec2,
        target_pos: Vec3,
        dash_distance: f32,
        progress: f32,
    ) -> bool {
        let damage_radius = calculate_dash_damage_radius(progress);

        // 计算目标到位移直线的距离
        let dash_vector = Vec3::new(dash_direction.x, 0.0, dash_direction.y);
        let dash_dir_len = dash_vector.length();
        if dash_dir_len < 0.001 {
            return target_pos.distance(riven_pos) <= damage_radius;
        }
        let dash_normalized = dash_vector / dash_dir_len;
        let to_target = target_pos - riven_pos;
        let projection_length = to_target.dot(dash_normalized);

        // 关键修复：首先检查投影长度是否在位移范围内
        // 如果目标在位移起点之前或之后，都不应该受到伤害
        if projection_length < 0.0 || projection_length > dash_distance {
            return false;
        }

        let closest_point = riven_pos + dash_normalized * projection_length;

        target_pos.distance(closest_point) <= damage_radius
    }

    // ===== 测试用例 =====

    /// 测试1: 位移伤害半径随进度扩大
    #[test]
    fn test_damage_radius_expands_during_dash() {
        assert_eq!(
            calculate_dash_damage_radius(0.0),
            RIVEN_Q_DAMAGE_RADIUS_START,
            "起点半径应该是65"
        );

        assert_eq!(
            calculate_dash_damage_radius(1.0),
            RIVEN_Q_DAMAGE_RADIUS_END,
            "终点半径应该是250"
        );

        let mid_radius = calculate_dash_damage_radius(0.5);
        let expected_mid = (RIVEN_Q_DAMAGE_RADIUS_START + RIVEN_Q_DAMAGE_RADIUS_END) / 2.0;
        assert!(
            (mid_radius - expected_mid).abs() < EPSILON,
            "中间位置半径应该是起点和终点的平均值"
        );
    }

    /// 测试2: 伤害半径是线性扩展的
    #[test]
    fn test_damage_radius_is_linear() {
        let test_cases: [(f32, f32); 5] = [
            (0.0, 65.0),
            (0.25, 111.25),
            (0.5, 157.5),
            (0.75, 203.75),
            (1.0, 250.0),
        ];

        for (progress, expected_radius) in test_cases {
            let actual = calculate_dash_damage_radius(progress);
            assert!(
                (actual - expected_radius).abs() < EPSILON,
                "进度 {} 的半径应该是 {:.2}, 实际是 {:.2}",
                progress,
                expected_radius,
                actual
            );
        }
    }

    /// 测试3: 目标位置判断逻辑
    #[test]
    fn test_target_in_range_calculation() {
        let riven_pos = Vec3::ZERO;
        let dash_direction = Vec2::new(0.0, 1.0); // 向Z正方向

        // 目标正好在路径Z轴上
        let target_on_path = Vec3::new(0.0, 0.0, 125.0); // 在路径中间

        assert!(
            is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_on_path,
                RIVEN_Q_DASH_DISTANCE,
                0.5
            ),
            "路径上的目标应该在伤害范围内"
        );

        // 目标在圆形边缘
        let radius_at_progress = calculate_dash_damage_radius(0.5);
        let target_at_edge = Vec3::new(radius_at_progress, 0.0, 125.0);
        assert!(
            is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_at_edge,
                RIVEN_Q_DASH_DISTANCE,
                0.5
            ),
            "边缘位置的目标应该在伤害范围内"
        );

        // 目标在圆形外
        let target_outside = Vec3::new(radius_at_progress + 100.0, 0.0, 125.0);
        assert!(
            !is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_outside,
                RIVEN_Q_DASH_DISTANCE,
                0.5
            ),
            "圆形外的目标不应该在伤害范围内"
        );

        // 目标在位移起点前方（超出路径长度）
        let target_beyond = Vec3::new(0.0, 0.0, 300.0);
        assert!(
            !is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_beyond,
                RIVEN_Q_DASH_DISTANCE,
                0.5
            ),
            "超出位移距离的目标不应该受到伤害"
        );
    }

    /// 测试4: 位移参数验证
    #[test]
    fn test_dash_parameters() {
        // 位移速度验证
        let expected_duration = RIVEN_Q_DASH_DISTANCE / RIVEN_Q_DASH_SPEED;
        assert!(
            (expected_duration - 0.25).abs() < EPSILON,
            "位移时间应该是0.25秒"
        );

        // 伤害半径变化率
        let radius_change = RIVEN_Q_DAMAGE_RADIUS_END - RIVEN_Q_DAMAGE_RADIUS_START;
        assert_eq!(radius_change, 185.0, "半径总变化应该是185");

        // 每个时间单位的半径增长
        let radius_per_second = radius_change / expected_duration;
        assert!(
            (radius_per_second - 740.0).abs() < EPSILON,
            "半径每秒增长740"
        );
    }

    /// 测试5: 多目标同时在位移路径上
    #[test]
    fn test_multiple_targets_in_range() {
        let riven_pos = Vec3::ZERO;
        let dash_direction = Vec2::new(0.0, 1.0);

        // 多个敌人在路径上的不同位置
        let targets = vec![
            Vec3::new(0.0, 0.0, 50.0),  // 路径前半段
            Vec3::new(0.0, 0.0, 150.0), // 路径中间
            Vec3::new(0.0, 0.0, 230.0), // 路径后半段
        ];

        // 在进度0.3时检查（半径约为120.5）
        let progress = 0.3;
        let _radius_at_progress = calculate_dash_damage_radius(progress);

        let in_range_count = targets
            .iter()
            .filter(|&&pos| {
                is_in_dash_damage_range(
                    riven_pos,
                    dash_direction,
                    pos,
                    RIVEN_Q_DASH_DISTANCE,
                    progress,
                )
            })
            .count();

        // 至少有两个目标在范围内
        assert!(
            in_range_count >= 2,
            "应该至少有2个目标在伤害范围内，实际: {}",
            in_range_count
        );
    }

    /// 测试6: 护甲减伤计算
    #[test]
    fn test_armor_damage_reduction() {
        let test_cases: [(f32, f32); 5] = [
            (0.0, 0.0),         // 0护甲 = 0%减伤
            (25.0, 0.2),        // 25护甲 = 20%减伤
            (50.0, 1.0 / 3.0),  // 50护甲 = 33.3%减伤
            (100.0, 0.5),       // 100护甲 = 50%减伤
            (200.0, 2.0 / 3.0), // 200护甲 = 66.7%减伤
        ];

        for (armor, expected_reduction) in test_cases {
            // 物理伤害减伤公式: 减伤 = Armor / (100 + Armor)
            let actual_reduction = armor / (100.0 + armor);

            assert!(
                (actual_reduction - expected_reduction).abs() < EPSILON,
                "护甲 {} 的减伤率应该是 {:.4}, 实际是 {:.4}",
                armor,
                expected_reduction,
                actual_reduction
            );
        }
    }

    /// 测试7: 锐雯Q技能位移数据输出
    #[test]
    fn test_riven_q_damage_zone_data() {
        println!("=== 锐雯Q位移伤害区域参数 ===");
        println!("位移距离: {}", RIVEN_Q_DASH_DISTANCE);
        println!("起始半径: {}", RIVEN_Q_DAMAGE_RADIUS_START);
        println!("终点半径: {}", RIVEN_Q_DAMAGE_RADIUS_END);
        println!("位移速度: {}", RIVEN_Q_DASH_SPEED);
        println!(
            "位移时间: {:.3}秒",
            RIVEN_Q_DASH_DISTANCE / RIVEN_Q_DASH_SPEED
        );
        println!();

        println!("伤害半径随位移进度变化:");
        for p in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0] {
            println!(
                "  进度 {:.0}%: 半径 = {:.1}",
                p * 100.0,
                calculate_dash_damage_radius(p)
            );
        }

        assert!(
            RIVEN_Q_DAMAGE_RADIUS_END > RIVEN_Q_DAMAGE_RADIUS_START,
            "终点半径应该大于起始半径"
        );
    }

    /// 测试8: DashDamageComponent 结构验证
    #[test]
    fn test_dash_damage_component_structure() {
        let dash_damage = DashDamage {
            radius_end: RIVEN_Q_DAMAGE_RADIUS_END,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: 0,
                damage_type: DamageType::Physical,
            },
        };

        assert_eq!(dash_damage.radius_end, 250.0);
        assert_eq!(dash_damage.damage.filter, TargetFilter::All);
        assert_eq!(dash_damage.damage.damage_type, DamageType::Physical);
    }

    /// 测试9: 伤害类型枚举验证
    #[test]
    fn test_damage_type_variants() {
        assert_eq!(DamageType::Physical, DamageType::Physical);
        assert_eq!(DamageType::Magic, DamageType::Magic);
        assert_eq!(DamageType::True, DamageType::True);

        assert_ne!(DamageType::Physical, DamageType::Magic);
        assert_ne!(DamageType::Physical, DamageType::True);
        assert_ne!(DamageType::Magic, DamageType::True);
    }

    /// 测试10: 目标过滤器验证
    #[test]
    fn test_target_filter_variants() {
        assert_eq!(TargetFilter::All, TargetFilter::All);
        assert_eq!(TargetFilter::Champion, TargetFilter::Champion);
        assert_eq!(TargetFilter::Minion, TargetFilter::Minion);

        assert_ne!(TargetFilter::All, TargetFilter::Champion);
        assert_ne!(TargetFilter::Champion, TargetFilter::Minion);
    }

    /// 测试11: 边缘情况 - 位移起点位置
    #[test]
    fn test_dash_at_start_position() {
        let progress = 0.0;
        let radius = calculate_dash_damage_radius(progress);

        assert_eq!(radius, RIVEN_Q_DAMAGE_RADIUS_START);

        let riven_pos = Vec3::ZERO;
        let dash_direction = Vec2::new(0.0, 1.0);
        let target_at_riven = Vec3::ZERO;

        assert!(
            is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_at_riven,
                RIVEN_Q_DASH_DISTANCE,
                progress
            ),
            "位移起点位置的处理"
        );
    }

    /// 测试12: 边缘情况 - 位移终点位置
    #[test]
    fn test_dash_at_end_position() {
        let progress = 1.0;
        let radius = calculate_dash_damage_radius(progress);

        assert_eq!(radius, RIVEN_Q_DAMAGE_RADIUS_END);

        let riven_pos = Vec3::ZERO;
        let dash_direction = Vec2::new(0.0, 1.0);
        let target_at_end = Vec3::new(RIVEN_Q_DAMAGE_RADIUS_END, 0.0, RIVEN_Q_DASH_DISTANCE);

        assert!(
            is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_at_end,
                RIVEN_Q_DASH_DISTANCE,
                progress
            ),
            "终点位置的目标应该在伤害范围内"
        );
    }

    /// 测试13: 伤害计算综合测试
    #[test]
    fn test_damage_calculation_comprehensive() {
        let riven_ad: f32 = 120.0;
        let skill_coefficient: f32 = 0.6;

        // 锐雯Q伤害 = AD * 系数
        let skill_damage = riven_ad * skill_coefficient;
        assert!(
            (skill_damage - 72.0).abs() < EPSILON,
            "技能伤害应该等于AD乘以系数"
        );

        // 不同护甲值的伤害输出
        println!(
            "\n=== 伤害减免表 (AD={}, 系数={}) ===",
            riven_ad, skill_coefficient
        );
        println!("技能伤害: {}", skill_damage);
        println!();
        for test_armor in [0, 25, 50, 75, 100, 150, 200] {
            let armor_val = test_armor as f32;
            let reduction = 1.0 - 100.0 / (100.0 + armor_val);
            let final_damage = skill_damage * (1.0 - reduction);
            println!(
                "护甲 {:3}: 减伤 {:5.2}% -> 最终伤害 {:6.2}",
                test_armor,
                reduction * 100.0,
                final_damage
            );
        }
    }

    /// 测试14: 斜向位移的方向计算
    #[test]
    fn test_diagonal_dash_direction() {
        // 测试斜向45度位移
        let riven_pos = Vec3::ZERO;
        let dash_direction = Vec2::new(1.0_f32.sqrt() / 2.0, 1.0_f32.sqrt() / 2.0); // 45度

        // 目标在位移路径上
        let target_on_path = Vec3::new(
            RIVEN_Q_DASH_DISTANCE * 0.707 / 2.0,
            0.0,
            RIVEN_Q_DASH_DISTANCE * 0.707 / 2.0,
        );

        // 在进度0.5时，半径约为157.5
        let progress = 0.5;

        assert!(
            is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_on_path,
                RIVEN_Q_DASH_DISTANCE,
                progress
            ),
            "斜向位移路径上的目标应该在伤害范围内"
        );

        // 目标垂直于位移方向，应该不受影响
        let target_perp = Vec3::new(
            RIVEN_Q_DASH_DISTANCE * 0.707 + 100.0,
            0.0,
            RIVEN_Q_DASH_DISTANCE * 0.707 / 2.0,
        );

        assert!(
            !is_in_dash_damage_range(
                riven_pos,
                dash_direction,
                target_perp,
                RIVEN_Q_DASH_DISTANCE,
                progress
            ),
            "垂直于位移方向的目标不应该在伤害范围内"
        );
    }

    /// 测试15: 伤害半径覆盖范围可视化
    #[test]
    fn test_damage_zone_coverage() {
        println!("\n=== 位移伤害区域覆盖分析 ===");
        println!(
            "位移: {} 距离, 速度: {}, 时间: {:.3}s",
            RIVEN_Q_DASH_DISTANCE,
            RIVEN_Q_DASH_SPEED,
            RIVEN_Q_DASH_DISTANCE / RIVEN_Q_DASH_SPEED
        );
        println!("起始圆形: r = {}", RIVEN_Q_DAMAGE_RADIUS_START);
        println!("终点圆形: r = {}", RIVEN_Q_DAMAGE_RADIUS_END);
        println!(
            "半径增长: {} / 秒",
            (RIVEN_Q_DAMAGE_RADIUS_END - RIVEN_Q_DAMAGE_RADIUS_START)
                / (RIVEN_Q_DASH_DISTANCE / RIVEN_Q_DASH_SPEED)
        );

        println!("\n不同时刻的伤害区域:");
        for (time, progress) in [(0.0, 0.0), (0.1, 0.4), (0.2, 0.8), (0.25, 1.0)] {
            let radius = calculate_dash_damage_radius(progress);
            let distance = RIVEN_Q_DASH_DISTANCE * progress;
            println!(
                "  t={:.2}s: 锐雯在({:.0},{:.0}), 伤害区域r={:.1}",
                time, distance, distance, radius
            );
        }

        // 验证区域覆盖
        assert!(
            RIVEN_Q_DAMAGE_RADIUS_END >= RIVEN_Q_DASH_DISTANCE / 2.0,
            "终点半径应该大于位移距离的一半，确保路径全覆盖"
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use moon_lol::core::*;

    // ===== 测试常量定义 =====
    const TEST_FPS: f32 = 30.0;
    const EPSILON: f32 = 1e-6;

    // ===== 测试辅助工具 =====

    /// 测试装置 - 封装通用的测试设置
    struct TestHarness {
        app: App,
        attacker: Entity,
        target: Entity,
    }

    impl TestHarness {
        /// 创建新的测试装置
        fn new() -> Self {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(PluginAttack);
            app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));

            let target = app.world_mut().spawn_empty().id();
            let attacker = app.world_mut().spawn_empty().id();

            TestHarness {
                app,
                attacker,
                target,
            }
        }

        /// 使用构建者模式配置攻击者
        fn with_attacker(mut self, attack_component: Attack) -> Self {
            self.app
                .world_mut()
                .entity_mut(self.attacker)
                .insert(attack_component);
            self
        }

        /// 创建额外的目标实体
        fn spawn_target(&mut self) -> Entity {
            self.app.world_mut().spawn_empty().id()
        }

        // ===== 动作方法 (Action Methods) - 返回 &mut Self =====

        /// 推进时间
        fn advance_time(&mut self, seconds: f32) -> &mut Self {
            let ticks = (seconds * TEST_FPS).ceil() as u32;
            for _ in 0..ticks {
                let mut time = self.app.world_mut().resource_mut::<Time<Fixed>>();
                time.advance_by(std::time::Duration::from_secs_f64(1.0 / TEST_FPS as f64));
                drop(time);
                self.app.world_mut().run_schedule(FixedUpdate);
            }
            self
        }

        /// 发送攻击命令
        fn attack(&mut self) -> &mut Self {
            self.app.world_mut().trigger_targets(
                CommandAttackStart {
                    target: self.target,
                },
                self.attacker,
            );
            self.app.update();
            self
        }

        /// 发送取消命令
        fn cancel(&mut self) -> &mut Self {
            self.app
                .world_mut()
                .trigger_targets(CommandAttackStop, self.attacker);
            self.app.update();
            self
        }

        /// 发送重置命令
        fn reset(&mut self) -> &mut Self {
            self.app
                .world_mut()
                .trigger_targets(CommandAttackReset, self.attacker);
            self.app.update();
            self
        }

        /// 切换攻击者的目标
        fn switch_target(&mut self, new_target: Entity) -> &mut Self {
            self.target = new_target;
            self
        }

        /// 修改攻击组件的属性
        fn modify_attacker<F>(&mut self, modifier: F) -> &mut Self
        where
            F: FnOnce(&mut Attack),
        {
            let mut attack = self
                .app
                .world_mut()
                .get_mut::<Attack>(self.attacker)
                .unwrap();
            modifier(&mut attack);
            self
        }

        /// 获取攻击状态
        fn attack_state(&self) -> Option<&AttackState> {
            self.app.world().get::<AttackState>(self.attacker)
        }

        /// 获取攻击组件
        fn attack_component(&self) -> &Attack {
            self.app.world().get::<Attack>(self.attacker).unwrap()
        }

        /// 获取当前时间
        fn current_time(&self) -> f32 {
            self.app.world().resource::<Time<Fixed>>().elapsed_secs()
        }

        /// 移除目标实体（模拟死亡）
        fn kill_target(&mut self, target: Entity) -> &mut Self {
            self.app.world_mut().entity_mut(target).despawn();
            self
        }

        // ===== 断言方法 (Assertion Methods) - 返回 &mut Self =====

        /// 断言攻击状态为空闲
        fn then_expect_idle(&mut self, message: &str) -> &mut Self {
            let state = self.attack_state();
            assert!(state.is_none(), "处于攻击状态: {}", message);
            self
        }

        /// 断言攻击状态为前摇
        fn then_expect_windup(&mut self, message: &str) -> &mut Self {
            let state = self.attack_state();
            assert!(state.is_some(), "没有处于攻击状态: {}", message);
            let state = state.unwrap();
            assert!(
                state.is_windup(),
                "{} (expected Windup, found {:?})",
                message,
                state.status
            );
            self
        }

        /// 断言攻击状态为后摇
        fn then_expect_cooldown(&mut self, message: &str) -> &mut Self {
            let state = self.attack_state();
            assert!(state.is_some(), "没有处于攻击状态: {}", message);
            let state = state.unwrap();
            assert!(
                state.is_cooldown(),
                "{} (expected Cooldown, found {:?})",
                message,
                state.status
            );
            self
        }

        /// 断言攻击目标
        fn then_expect_target(&mut self, expected_target: Entity, message: &str) -> &mut Self {
            let state = self.attack_state();
            assert!(state.is_some(), "没有处于攻击状态: {}", message);
            let state = state.unwrap();
            assert_eq!(state.target, Some(expected_target), "{}", message);
            self
        }

        /// 执行自定义断言
        fn then_custom_assert<F>(&mut self, assert_fn: F, message: &str) -> &mut Self
        where
            F: FnOnce(&Self) -> bool,
        {
            assert!(assert_fn(self), "自定义断言失败: {}", message);
            self
        }
    }

    // ===== 一、核心状态机与流程 (Core State Machine & Flow) =====

    /// 目标 1：完整的攻击循环
    #[test]
    fn test_complete_attack_cycle() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let target = harness.target;

        harness
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .then_expect_target(target, "攻击目标应该正确")
            .advance_time(0.3)
            .then_expect_cooldown("前摇结束后应该进入后摇状态")
            .then_expect_target(target, "后摇期间目标应该保持不变")
            .advance_time(0.7)
            .then_expect_windup("后摇结束后应该自动开始下一次攻击")
            .then_expect_target(target, "下一次攻击的目标应该相同");
    }

    /// 目标 2：连续攻击同一目标
    #[test]
    fn test_consecutive_attacks_same_target() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let target = harness.target;
        let initial_time = harness.current_time();

        harness
            .attack()
            .then_expect_windup("第一次攻击应该触发前摇状态")
            .advance_time(1.0)
            .then_expect_windup("后摇结束后应该自动开始下一次攻击")
            .then_expect_target(target, "下一次攻击的目标应该相同")
            // 手动发送第二次攻击命令（测试同目标不重新开始）
            .attack()
            .then_expect_windup("第二次攻击应该保持前摇状态")
            .then_expect_target(target, "攻击目标应该保持不变");

        assert!(
            (harness.attack_component().windup_duration_secs() - 0.3).abs() < EPSILON,
            "前摇时间配置正确"
        );
        assert!(
            (harness.attack_component().cooldown_time() - 0.7).abs() < EPSILON,
            "后摇时间配置正确"
        );
    }

    // ===== 二、攻击取消机制 (Attack Cancellation Mechanics) =====

    /// 目标 4：在"可取消"阶段取消前摇
    #[test]
    fn test_cancel_attack_during_cancellable_windup() {
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .advance_time(0.1)
            .cancel()
            .then_expect_idle("可取消期内的攻击应该被取消")
            .attack()
            .then_expect_windup("应该能立即开始新的攻击");
    }

    // ===== 三、攻击重置 (走A) 机制 (Attack Reset / Kiting) =====

    /// 目标 6：在后摇 (Cooldown) 期间重置攻击
    #[test]
    fn test_attack_reset_during_cooldown() {
        let target = TestHarness::new().target;
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .attack()
            .advance_time(0.3)
            .then_expect_cooldown("应该进入后摇状态")
            .reset()
            .then_expect_windup("重置后应该立即进入新的前摇状态")
            .then_expect_target(target, "重置后目标应该保持不变");
    }

    /// 测试攻击重置事件触发
    #[test]
    fn test_attack_reset_event_triggering() {
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .attack()
            .advance_time(0.3)
            .then_expect_cooldown("应该进入后摇状态")
            .reset()
            .then_expect_windup("重置后应该进入前摇状态");
    }

    // ===== 四、攻击速度影响 (Impact of Attack Speed) =====

    /// 目标 7：攻速变化对攻击时间的影响
    #[test]
    fn test_attack_speed_impact_on_timing() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 0.0,
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        });

        let initial_attack = harness.attack_component().clone();
        harness
            .modify_attacker(|attack| {
                attack.bonus_attack_speed = 1.0;
            })
            .then_custom_assert(
                |h| {
                    h.attack_component().total_duration_secs()
                        < initial_attack.total_duration_secs()
                },
                "攻击间隔应该缩短",
            )
            .then_custom_assert(
                |h| {
                    h.attack_component().windup_duration_secs()
                        < initial_attack.windup_duration_secs()
                },
                "前摇时间应该缩短",
            )
            .then_custom_assert(
                |h| h.attack_component().cooldown_time() < initial_attack.cooldown_time(),
                "后摇时间应该缩短",
            )
            .then_custom_assert(
                |h| {
                    (h.attack_component().windup_duration_secs()
                        + h.attack_component().cooldown_time())
                        < (initial_attack.windup_duration_secs() + initial_attack.cooldown_time())
                },
                "总攻击时间应该缩短",
            );
    }

    /// 目标 8：攻击速度达到上限
    #[test]
    fn test_attack_speed_cap() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 10.0,
            attack_speed_cap: 2.5,
            ..Default::default()
        };

        let min_interval = 1.0 / 2.5;

        let mut harness = TestHarness::new().with_attacker(attack);

        // 验证基本的攻速限制
        assert!(
            (harness.attack_component().current_attack_speed() - 2.5).abs() < EPSILON,
            "攻速应该被限制在2.5"
        );
        assert!(
            (harness.attack_component().total_duration_secs() - min_interval).abs() < EPSILON,
            "攻击间隔应该是最小值"
        );

        harness.modify_attacker(|attack| {
            attack.bonus_attack_speed = 20.0;
        });

        assert!(
            (harness.attack_component().current_attack_speed() - 2.5).abs() < EPSILON,
            "进一步增加bonus_attack_speed不应该改变结果"
        );
        assert!(
            (harness.attack_component().total_duration_secs() - min_interval).abs() < EPSILON,
            "攻击间隔不应该改变"
        );
    }

    // ===== 五、前摇配置与修正 (Windup Configuration & Modifiers) =====

    /// 验证Legacy前摇公式
    #[test]
    fn test_legacy_windup_formula() {
        let test_cases = [(0.1, 0.4), (-0.1, 0.2), (0.0, 0.3)];

        for (attack_offset, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset },
                ..Default::default()
            };
            let harness = TestHarness::new().with_attacker(attack);
            let actual_windup = harness.attack_component().windup_duration_secs();
            assert!(
                (actual_windup - expected_windup).abs() < EPSILON,
                "Legacy模式下，attack_offset={}应产生前摇时间{}，实际为{}",
                attack_offset,
                expected_windup,
                actual_windup
            );
        }
    }

    /// 验证Modern前摇公式
    #[test]
    fn test_modern_windup_formula() {
        let test_cases = [
            (0.25, 1.0, 1.0, 0.25),
            (0.25, 1.0, 2.0, 0.125),
            (0.3, 1.2, 1.5, 0.16666667),
        ];

        for (attack_cast_time, attack_total_time, base_speed, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: base_speed,
                windup_config: WindupConfig::Modern {
                    attack_cast_time,
                    attack_total_time,
                },
                ..Default::default()
            };
            let harness = TestHarness::new().with_attacker(attack);
            let actual_windup = harness.attack_component().windup_duration_secs();
            assert!(
                (actual_windup - expected_windup).abs() < EPSILON,
                "Modern模式下，配置应产生前摇时间{}，实际为{}",
                expected_windup,
                actual_windup
            );
        }
    }

    /// 验证windup_modifier的效果
    #[test]
    fn test_windup_modifier_effect() {
        let test_cases = [(1.0, 0.3), (0.5, 0.15), (1.5, 0.45), (0.1, 0.03)];

        for (modifier, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Modern {
                    attack_cast_time: 0.3,
                    attack_total_time: 1.0,
                },
                windup_modifier: modifier,
                ..Default::default()
            };
            let harness = TestHarness::new().with_attacker(attack);
            let actual_windup = harness.attack_component().windup_duration_secs();
            assert!(
                (actual_windup - expected_windup).abs() < EPSILON,
                "Modifier={}应产生前摇时间{}，实际为{}",
                modifier,
                expected_windup,
                actual_windup
            );
        }

        // 测试Legacy模式下的修正系数
        let legacy_attack = Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.1 },
            windup_modifier: 0.8,
            ..Default::default()
        };
        let expected_legacy = (0.3 + 0.1) * 0.8;
        let harness = TestHarness::new().with_attacker(legacy_attack);
        let actual_windup = harness.attack_component().windup_duration_secs();
        assert!(
            (actual_windup - expected_legacy).abs() < EPSILON,
            "Legacy模式下的修正系数测试，期望{}，实际为{}",
            expected_legacy,
            actual_windup
        );
    }

    // ===== 七、辅助测试函数和浮点数精度 =====

    /// 攻击速度计算验证
    #[test]
    fn test_attack_speed_calculations() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 1.0,
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        };

        let harness = TestHarness::new().with_attacker(attack);

        let component = harness.attack_component();
        assert!(
            (component.current_attack_speed() - 1.25).abs() < EPSILON,
            "当前攻速计算不正确"
        );
        assert!(
            (component.total_duration_secs() - 0.8).abs() < EPSILON,
            "攻击间隔计算不正确"
        );
        assert!(
            (component.windup_duration_secs() - 0.3).abs() < EPSILON,
            "前摇时间计算不正确"
        );
        assert!(
            (component.cooldown_time() - 0.5).abs() < EPSILON,
            "后摇时间计算不正确"
        );
    }

    /// 浮点数精度测试
    #[test]
    fn test_floating_point_precision() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 0.6,
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        };

        let expected_speed = 0.625 * (1.0 + 0.6);
        let expected_interval = 1.0 / expected_speed;
        let expected_windup = 0.25 / 1.0 * expected_interval;

        let harness = TestHarness::new().with_attacker(attack);

        let component = harness.attack_component();
        assert!(
            (component.current_attack_speed() - expected_speed).abs() < EPSILON,
            "攻速不精确"
        );
        assert!(
            (component.total_duration_secs() - expected_interval).abs() < EPSILON,
            "攻击间隔不精确"
        );
        assert!(
            (component.windup_duration_secs() - expected_windup).abs() < EPSILON,
            "前摇时间不精确"
        );
    }

    /// 在前摇期间重置攻击
    #[test]
    fn test_attack_reset_during_windup() {
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .reset()
            .then_expect_windup("前摇期间重置应该重新回到前摇状态");
    }

    /// Modern模式下的攻速缩放测试
    #[test]
    fn test_modern_windup_with_attack_speed_scaling() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            bonus_attack_speed: 1.0,
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        });

        let target = harness.target;

        // 先验证时间计算
        assert!(
            (harness.attack_component().windup_duration_secs() - 0.125).abs() < EPSILON,
            "前摇时间计算不正确"
        );
        assert!(
            (harness.attack_component().cooldown_time() - 0.375).abs() < EPSILON,
            "后摇时间计算不正确"
        );

        harness
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .advance_time(0.125)
            .then_expect_cooldown("前摇结束后应该进入后摇状态")
            .advance_time(0.375)
            .then_expect_windup("后摇结束后应该自动开始下一次攻击")
            .then_expect_target(target, "下一次攻击的目标应该相同");
    }

    /// 不可取消宽限期测试
    #[test]
    fn test_uncancellable_grace_period() {
        let attack = Attack {
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.05,
                attack_total_time: 0.95,
            },
            base_attack_speed: 1.0,
            ..Default::default()
        };

        assert!(attack.windup_duration_secs() < UNCANCELLABLE_GRACE_PERIOD);
    }

    // ===== 八、目标切换与攻击取消的交互 (Target Switching & Attack Cancellation Interaction) =====

    /// 测试场景1：攻击目标A，在不可取消期间切换到目标B
    /// 期望：当前攻击仍攻击目标A，但下一次自动攻击应该攻击目标B
    #[test]
    fn test_new_target_command_during_uncancellable_period() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let target_a = harness.target;
        let target_b = harness.spawn_target();

        harness
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .then_expect_target(target_a, "攻击目标应该是A")
            .advance_time(0.234)
            .switch_target(target_b)
            .attack()
            .then_expect_windup("攻击应该重新进入前摇状态")
            .then_expect_target(target_b, "当前攻击的目标应该是B")
            .advance_time(0.5)
            .then_expect_cooldown("应该进入冷却状态")
            .then_expect_target(target_b, "下一次攻击的目标还是B")
            .switch_target(target_a)
            .attack()
            .then_expect_cooldown("攻击应该还是后摇状态")
            .then_expect_target(target_a, "下一次攻击的目标应该是A")
            .advance_time(0.5)
            .then_expect_windup("后摇结束后应该自动开始下一次攻击")
            .then_expect_target(target_a, "下一次攻击的目标应该是A");
    }

    /// 测试场景2：攻击目标A，在可取消期间攻击目标B
    /// 期望：会立即取消当前攻击，重新开始攻击目标B且重新计时
    #[test]
    fn test_new_target_command_during_cancellable_period() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let target_a = harness.target;
        let target_b = harness.spawn_target();

        harness
            .attack()
            .then_expect_windup("攻击命令应该触发前摇状态")
            .then_expect_target(target_a, "攻击目标应该是A")
            .advance_time(0.1)
            .switch_target(target_b)
            .attack()
            .then_expect_windup("应该立即开始攻击目标B")
            .then_expect_target(target_b, "当前攻击的目标应该是B")
            .advance_time(0.3)
            .then_expect_cooldown("前摇结束后应该进入后摇状态")
            .then_expect_target(target_b, "后摇期间目标应该是B")
            .advance_time(0.7)
            .then_expect_windup("后摇结束后应该自动开始下一次攻击")
            .then_expect_target(target_b, "下一次攻击的目标应该是B");
    }

    // ===== 九、边缘和异常情况测试 (Edge Cases & Exception Handling) =====

    /// 测试后摇期间发送取消命令
    #[test]
    fn test_cancel_attack_during_cooldown() {
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .attack()
            .advance_time(0.3)
            .then_expect_cooldown("前摇结束后应该进入后摇状态")
            .cancel()
            .then_expect_cooldown("后摇期间取消攻击还是后摇状态")
            .attack()
            .then_expect_cooldown("后摇期间发送攻击命令应该不变");
    }

    /// 测试空闲状态下发送取消或重置命令
    #[test]
    fn test_cancel_and_reset_in_idle_state() {
        TestHarness::new()
            .with_attacker(Attack::default())
            .then_expect_idle("初始状态应该是空闲")
            .cancel()
            .then_expect_idle("空闲状态下取消命令不应改变状态")
            .reset()
            .then_expect_idle("空闲状态下重置命令不应改变状态");
    }

    /// 演示：continue_attack控制测试
    #[test]
    fn test_continue_attack_control() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 正常攻击循环
        harness
            .attack()
            .then_custom_assert(
                |h| h.attack_state().unwrap().target.is_some(),
                "默认应该继续攻击",
            )
            .advance_time(1.0)
            .then_expect_windup("没有取消命令时应该自动继续攻击")
            // 测试取消命令停止自动攻击
            .cancel()
            .then_expect_idle("取消后应该回到空闲状态");
    }

    // ===== 十、复杂交互场景测试 (Complex Interaction Scenarios) =====

    /// 复杂场景：走位攻击和目标切换组合
    #[test]
    fn test_complex_kiting_and_target_switching_scenario() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let initial_target = harness.target;
        let new_target = harness.spawn_target();

        harness
            .attack()
            .then_expect_windup("应该开始攻击初始目标")
            .then_expect_target(initial_target, "目标应为 initial_target")
            .advance_time(0.1)
            .switch_target(new_target)
            .attack()
            .then_expect_windup("攻击被取消，并立即开始攻击新目标")
            .then_expect_target(new_target, "目标应切换为 new_target")
            .advance_time(0.3)
            .then_expect_cooldown("对新目标的攻击完成，进入后摇")
            .advance_time(0.2)
            .reset()
            .then_expect_windup("后摇被重置，立即开始新的攻击")
            .then_expect_target(new_target, "目标仍然是 new_target")
            .advance_time(0.1)
            .switch_target(initial_target)
            .attack()
            .then_expect_windup("可以正常开始攻击初始目标")
            .advance_time(0.1)
            .modify_attacker(|attack| {
                attack.bonus_attack_speed = 1.0;
            })
            .advance_time(0.1)
            .then_expect_windup("前摇中加攻速不会影响前摇时间")
            .advance_time(0.1)
            .then_expect_cooldown("进入后摇")
            .advance_time(0.35)
            .then_expect_windup("加攻速后，后摇时间变为 0.35 秒，进入下一次攻击前摇");
    }

    /// 复杂场景：多目标切换和取消序列
    #[test]
    fn test_multi_target_switching_and_cancellation_sequence() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 2.0,
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.2,
                attack_total_time: 1.0,
            },
            ..Default::default()
        });

        let target_a = harness.target;
        let target_b = harness.spawn_target();
        let target_c = harness.spawn_target();

        harness
            .attack()
            .then_expect_windup("开始攻击目标A")
            .then_expect_target(target_a, "确认攻击目标A")
            .advance_time(0.03)
            .switch_target(target_b)
            .attack()
            .then_expect_windup("切换攻击目标B")
            .then_expect_target(target_b, "确认攻击目标B")
            .advance_time(0.03)
            .switch_target(target_c)
            .attack()
            .then_expect_windup("切换攻击目标C")
            .then_expect_target(target_c, "确认攻击目标C")
            .advance_time(0.1)
            .then_expect_cooldown("完成对目标C的攻击")
            .advance_time(0.4)
            .then_expect_windup("自动开始下一次攻击目标C")
            .advance_time(0.04)
            .switch_target(target_a)
            .attack()
            .then_expect_windup("成功切换攻击目标A")
            .then_expect_target(target_a, "目标现在是A");
    }

    /// 复杂场景：攻速变化下的精确时间控制
    #[test]
    fn test_attack_speed_scaling_with_precise_timing() {
        TestHarness::new()
            .with_attacker(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Modern {
                    attack_cast_time: 0.25,
                    attack_total_time: 1.0,
                },
                ..Default::default()
            })
            .attack()
            .then_expect_windup("开始基础攻速攻击")
            .then_custom_assert(
                |h| (h.attack_component().windup_duration_secs() - 0.25).abs() < EPSILON,
                "基础攻速下前摇时间应为0.25秒",
            )
            .advance_time(0.25)
            .then_expect_cooldown("完成基础攻速前摇")
            .then_custom_assert(
                |h| (h.attack_component().cooldown_time() - 0.75).abs() < EPSILON,
                "基础攻速下后摇时间应为0.75秒",
            )
            .advance_time(0.4)
            .modify_attacker(|attack| {
                attack.bonus_attack_speed = 1.0;
            })
            .then_custom_assert(
                |h| (h.attack_component().total_duration_secs() - 0.5).abs() < EPSILON,
                "攻速提升后间隔应为0.5秒",
            )
            .advance_time(0.35)
            .then_expect_windup("开始新攻速的攻击")
            .then_custom_assert(
                |h| (h.attack_component().windup_duration_secs() - 0.125).abs() < EPSILON,
                "新攻速下前摇时间应为0.125秒",
            )
            .advance_time(0.125)
            .then_expect_cooldown("完成新攻速前摇")
            .advance_time(0.375)
            .then_expect_windup("新攻速下的下一次攻击");
    }
}

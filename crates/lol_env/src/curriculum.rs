//! 课程学习调度器模块
//! 
//! 该模块定义了智能体的课程学习阶段、配置及调度逻辑。
//! 通过逐步提高任务难度，帮助智能体更快收敛。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// 课程学习阶段
// ============================================================================

/// 课程学习阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurriculumPhase {
    /// 第一课：学补刀，小兵血量从低到高
    LastHitTraining,
    /// 第二课：补刀优先 + 消耗对手
    HarassTraining,
}

// ============================================================================
// 课程学习配置（重导出自 lol_rl_protocol 作为单一事实来源）
// ============================================================================

pub use lol_rl_protocol::CurriculumConfig;

// ============================================================================
// 课程学习调度器
// ============================================================================

/// 课程学习调度器，管理训练阶段和相关参数
#[derive(Debug, Clone)]
pub struct CurriculumScheduler {
    config: CurriculumConfig,
    phase: CurriculumPhase,
    current_hp_scale: f32,
    current_harass_coef: f32,
}

impl CurriculumScheduler {
    /// 创建新的调度器
    pub fn new(config: CurriculumConfig) -> Self {
        Self {
            current_hp_scale: config.hp_scale_start,
            config,
            phase: CurriculumPhase::LastHitTraining,
            current_harass_coef: 0.0,
        }
    }

    /// 更新调度器状态
    pub fn tick(&mut self, iter: usize, avg_cs_per_episode: f32) {
        match self.phase {
            CurriculumPhase::LastHitTraining => {
                // 第一课期间：hp_scale 线性增长
                let progress = (iter as f32 / self.config.phase1_iterations as f32).min(1.0);
                self.current_hp_scale = self.config.hp_scale_start
                    + (self.config.hp_scale_end - self.config.hp_scale_start) * progress;

                // 检查是否可以进入第二课
                if self.current_hp_scale >= self.config.hp_scale_end
                    && avg_cs_per_episode >= self.config.phase2_cs_threshold
                {
                    self.phase = CurriculumPhase::HarassTraining;
                    self.current_hp_scale = 1.0;
                    self.current_harass_coef = self.config.harass_coef;
                }
            }
            CurriculumPhase::HarassTraining => {
                // 第二课：hp_scale 保持 1.0，开启消耗奖励
                self.current_hp_scale = 1.0;
                self.current_harass_coef = self.config.harass_coef;
            }
        }
    }

    /// 获取当前阶段
    pub fn phase(&self) -> &CurriculumPhase {
        &self.phase
    }

    /// 获取当前小兵血量缩放
    pub fn minion_hp_scale(&self) -> f32 {
        self.current_hp_scale
    }

    /// 获取当前英雄伤害奖励系数
    pub fn harass_coef(&self) -> f32 {
        self.current_harass_coef
    }

    /// 获取当前补刀奖励
    pub fn cs_reward(&self) -> f32 {
        self.config.cs_reward
    }

    /// 获取当前攻击小兵未补到刀的惩罚
    pub fn attack_no_cs_penalty(&self) -> f32 {
        self.config.attack_no_cs_penalty
    }

    /// 获取当前阶段名称
    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            CurriculumPhase::LastHitTraining => "LastHitTraining",
            CurriculumPhase::HarassTraining => "HarassTraining",
        }
    }

    /// 获取日志摘要信息
    pub fn summary(&self) -> String {
        format!(
            "Phase: {}, HP Scale: {:.2}, Harass Coef: {:.2}, CS Reward: {:.2}, Penalty: {:.2}",
            self.phase_name(),
            self.current_hp_scale,
            self.current_harass_coef,
            self.config.cs_reward,
            self.config.attack_no_cs_penalty
        )
    }

    /// 导出通用的前端实时遥测数据（零业务硬编码契约）
    pub fn to_telemetry(&self, avg_cs: f32) -> lol_rl_protocol::CurriculumTelemetry {
        let (phase_index, phase_name, progress, transition_condition) = match self.phase {
            CurriculumPhase::LastHitTraining => {
                let prog = if self.config.hp_scale_end > self.config.hp_scale_start {
                    ((self.current_hp_scale - self.config.hp_scale_start)
                        / (self.config.hp_scale_end - self.config.hp_scale_start))
                        .clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let cond = format!(
                    "小兵血量达100%且平均CS≥{:.1} (当前CS: {:.2})",
                    self.config.phase2_cs_threshold, avg_cs
                );
                (0, "第一课: 基础补刀".to_string(), prog, Some(cond))
            }
            CurriculumPhase::HarassTraining => (
                1,
                "第二课: 补刀优先+消耗对手".to_string(),
                1.0,
                Some("课程已全部解锁".to_string()),
            ),
        };

        let mut parameters = std::collections::HashMap::new();
        parameters.insert("小兵血量比例 (hp_scale)".to_string(), self.current_hp_scale);
        parameters.insert("补刀奖励 (cs_reward)".to_string(), self.config.cs_reward);
        parameters.insert("无效攻击惩罚 (penalty)".to_string(), self.config.attack_no_cs_penalty);
        parameters.insert("消耗对手系数 (harass)".to_string(), self.current_harass_coef);

        lol_rl_protocol::CurriculumTelemetry {
            phase_index,
            total_phases: 2,
            phase_name,
            all_phase_names: vec![
                "第一课: 基础补刀".to_string(),
                "第二课: 补刀优先+消耗对手".to_string(),
            ],
            progress,
            transition_condition,
            parameters,
        }
    }
}

// ============================================================================
// 奖励配置 Resource
// ============================================================================

/// 课程奖励配置，作为 Bevy Resource 注入到 World 中
#[derive(Resource, Debug, Clone)]
pub struct CurriculumRewardConfig {
    pub cs_reward: f32,
    pub attack_no_cs_penalty: f32,
    pub harass_coef: f32,
    pub minion_hp_scale: f32,
}

impl Default for CurriculumRewardConfig {
    fn default() -> Self {
        Self {
            cs_reward: 1.0,
            attack_no_cs_penalty: 0.1,
            harass_coef: 0.0,
            minion_hp_scale: 1.0,
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_config_default() {
        let config = CurriculumConfig::default();
        assert_eq!(config.hp_scale_start, 0.05);
        assert_eq!(config.hp_scale_end, 1.0);
        assert_eq!(config.phase1_iterations, 200);
        assert_eq!(config.phase2_cs_threshold, 2.0);
        assert_eq!(config.harass_coef, 0.3);
        assert_eq!(config.cs_reward, 1.0);
        assert_eq!(config.attack_no_cs_penalty, 0.1);
    }

    #[test]
    fn test_tick_interpolation_and_phase_transition() {
        let mut config = CurriculumConfig::default();
        config.phase1_iterations = 100;
        config.hp_scale_start = 0.0;
        config.hp_scale_end = 1.0;
        config.phase2_cs_threshold = 5.0;

        let mut scheduler = CurriculumScheduler::new(config);

        // 初始状态
        assert_eq!(scheduler.phase(), &CurriculumPhase::LastHitTraining);
        assert_eq!(scheduler.minion_hp_scale(), 0.0);
        assert_eq!(scheduler.harass_coef(), 0.0);

        // 第 50 次迭代，一半进度
        scheduler.tick(50, 0.0);
        assert_eq!(scheduler.minion_hp_scale(), 0.5);
        assert_eq!(scheduler.phase(), &CurriculumPhase::LastHitTraining);

        // 第 100 次迭代，进度满，但 CS 不够
        scheduler.tick(100, 4.9);
        assert_eq!(scheduler.minion_hp_scale(), 1.0);
        assert_eq!(scheduler.phase(), &CurriculumPhase::LastHitTraining);

        // 第 110 次迭代，CS 达标，切换到第二阶段
        scheduler.tick(110, 5.1);
        assert_eq!(scheduler.phase(), &CurriculumPhase::HarassTraining);
        assert_eq!(scheduler.minion_hp_scale(), 1.0);
        assert_eq!(scheduler.harass_coef(), 0.3);
    }
}

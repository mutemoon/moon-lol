use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::ActionSchema;
use crate::frames::ObsFeaturePayload;
use crate::obs::{ObsSchema, ObsValueNode};
use crate::reward::{RewardFormulaSpec, RewardItem};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PolicyItem {
    pub action_id: usize,
    pub action: String,
    /// Mask 后的有效采样概率
    pub prob: f32,
    /// 未 Mask 的网络原始 Softmax 概率
    #[serde(default)]
    pub raw_prob: f32,
    /// 当前步是否被 Action Mask 屏蔽
    #[serde(default)]
    pub is_masked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ActionBranchDisplay {
    Categorical {
        name: String,
        items: Vec<PolicyItem>,
    },
    Continuous {
        name: String,
        means: Vec<f32>,
        labels: Vec<String>,
    },
    UnitSelection {
        name: String,
        obs_entity_name: String,
        items: Vec<PolicyItem>,
    },
    Struct {
        name: String,
        fields: Vec<ActionBranchDisplay>,
    },
}

/// 可视化策略展示：按真实动作空间区分（连续/混合/结构化多头环境）。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PolicyDisplay {
    /// 离散动作空间：每个离散动作的概率分布。
    Discrete(Vec<PolicyItem>),
    /// 混合动作空间（连续移动 + 离散攻击）：连续均值 + 攻击概率。
    Hybrid {
        move_x: f32,
        move_z: f32,
        attack_prob: f32,
        #[serde(default)]
        raw_attack_prob: f32,
        #[serde(default)]
        is_attack_masked: bool,
    },
    /// 增强混合动作空间（多维连续偏移均值 + 多离散动作概率分布）
    HybridMulti {
        continuous_means: Vec<f32>,
        discrete_probs: Vec<PolicyItem>,
    },
    /// 通用结构化动作空间展示（支持任意组合嵌套的分类头、连续头、单位选择头）
    Structured(Vec<ActionBranchDisplay>),
}

// ── Visual subprocess protocol ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualObsFrame {
    pub step: usize,
    pub obs: ObsFeaturePayload,
    pub reward: f32,
    /// 本局从开始累积到当前步的总奖励（每次对局重置后归零）。
    #[serde(default)]
    pub episode_reward: f32,
    pub reward_breakdown: Vec<RewardItem>,
    pub policy: PolicyDisplay,
    pub terminated: bool,
    pub truncated: bool,
    #[serde(default)]
    pub self_alive: bool,
    #[serde(default)]
    pub target_alive: bool,
    #[serde(default)]
    pub fiora_alive: bool,
    #[serde(default)]
    pub riven_alive: bool,
    pub reward_formula: Option<RewardFormulaSpec>,
    pub reward_variables: Option<HashMap<String, f32>>,
    /// 策略真实输入的观测向量。
    #[serde(default)]
    pub obs_vector: Vec<f32>,
    /// 观测向量每一维的简要说明。
    #[serde(default)]
    pub obs_labels: Vec<String>,
    /// 结构化 AST 观测树（供前端动态长度、可折叠、可嵌套展示）。
    #[serde(default)]
    pub obs_tree: Option<Vec<ObsValueNode>>,
    /// 当前是否处于暂停状态。
    #[serde(default)]
    pub is_paused: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualOutFrame {
    Ready {
        checkpoint_path: String,
        env_name: String,
        env_max_steps: usize,
        action_labels: Vec<String>,
        #[serde(default)]
        obs_schema: Option<ObsSchema>,
        #[serde(default)]
        action_schema: Option<ActionSchema>,
    },
    Frame(VisualObsFrame),
    Paused(bool),
    Log {
        level: String,
        message: String,
    },
    Exited {
        code: Option<i32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VisualInFrame {
    Reset,
    Pause,
    Resume,
    StepOnce,
    StepWithAction { action_id: usize },
    SetAutoPause(bool),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obs::ObsNode;

    #[test]
    fn test_visual_frames_bincode_roundtrip() {
        let schema = ObsSchema::new(vec![ObsNode::structure(
            "hero",
            vec![
                ObsNode::categorical("role", 4, 12),
                ObsNode::scalar("hp_pct", 0.0, 1.0),
            ],
        )]);
        let labels = schema.to_dim_labels();
        let tree = schema.decode_tree(&[1.0; 2]);

        let frame = VisualOutFrame::Ready {
            checkpoint_path: "test.safetensors".into(),
            env_name: "SoloV0".into(),
            env_max_steps: 100,
            action_labels: vec!["a1".into()],
            obs_schema: Some(schema),
            action_schema: None,
        };
        let bytes = bincode::serialize(&frame).expect("bincode serialize Ready");
        let decoded: VisualOutFrame =
            bincode::deserialize(&bytes).expect("bincode deserialize Ready");
        match decoded {
            VisualOutFrame::Ready { obs_schema, .. } => {
                assert!(obs_schema.is_some());
            }
            _ => panic!("Expected Ready"),
        }

        let obs_frame = VisualObsFrame {
            step: 1,
            obs: ObsFeaturePayload::default(),
            reward: 1.0,
            episode_reward: 10.0,
            reward_breakdown: vec![],
            policy: PolicyDisplay::Discrete(vec![]),
            terminated: false,
            truncated: false,
            self_alive: true,
            target_alive: true,
            fiora_alive: true,
            riven_alive: true,
            reward_formula: None,
            reward_variables: None,
            obs_vector: vec![1.0; 2],
            obs_labels: labels,
            obs_tree: Some(tree),
            is_paused: false,
        };
        let out_frame = VisualOutFrame::Frame(obs_frame);
        let bytes2 = bincode::serialize(&out_frame).expect("bincode serialize Frame");
        let decoded2: VisualOutFrame =
            bincode::deserialize(&bytes2).expect("bincode deserialize Frame");
        match decoded2 {
            VisualOutFrame::Frame(f) => {
                assert_eq!(f.obs_tree.as_ref().unwrap().len(), 1);
            }
            _ => panic!("Expected Frame"),
        }
    }
}

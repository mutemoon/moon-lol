use serde::{Deserialize, Serialize};

pub const ENV_SOLO_V0: &str = "SoloV0";
pub const ENV_FIORA_V2: &str = "FioraV2";
pub const ENV_FIORA_V1: &str = "FioraV1";
pub const ENV_FIORA_V0: &str = "FioraV0";

/// 环境自带的训练超参数规范（作为环境默认超参数的唯一真实来源）。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EnvTrainingParams {
    pub lr: f32,
    pub gamma: f32,
    pub gae_lambda: f32,
    pub clip_eps: f32,
    pub ppo_epochs: usize,
    pub hidden_dim: usize,
    pub rollout_steps_per_env: usize,
    pub total_iterations: usize,
}

/// 环境规范与展示元数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub default_params: EnvTrainingParams,
}

pub const ENV_SOLO_V0_SPEC: EnvSpec = EnvSpec {
    name: ENV_SOLO_V0,
    label: "剑姬 vs 瑞雯 (Solo 1v1 自博弈)",
    tag: "SoloV0",
    description: "单神经网络通过 role_id (0:剑姬, 1:瑞雯) 自博弈对抗，对称零和奖励与自我中心化全技能对决",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 64,
        rollout_steps_per_env: 160,
        total_iterations: 500,
    },
};

pub const ENV_FIORA_V2_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V2,
    label: "全技能实战 (V2)",
    tag: "V2",
    description: "基于 OpenAI Five 统一结构化 Modifier 槽位与通用表征架构的全技能微操环境",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 64,
        rollout_steps_per_env: 160,
        total_iterations: 300,
    },
};

pub const ENV_FIORA_V1_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V1,
    label: "真实移动 (V1)",
    tag: "V1",
    description: "模拟真实微操移动与普攻破绽打击，连续空间离散化动作",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 80,
    },
};

pub const ENV_FIORA_V0_SPEC: EnvSpec = EnvSpec {
    name: ENV_FIORA_V0,
    label: "瞬移站位 (V0)",
    tag: "V0",
    description: "简化版瞬移站位打弱点机制，快速收敛验证基础 PPO 策略",
    default_params: EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 4,
        hidden_dim: 64,
        rollout_steps_per_env: 80,
        total_iterations: 50,
    },
};

pub const AVAILABLE_ENVS: &[EnvSpec] = &[
    ENV_SOLO_V0_SPEC,
    ENV_FIORA_V2_SPEC,
    ENV_FIORA_V1_SPEC,
    ENV_FIORA_V0_SPEC,
];

pub fn get_env_spec(name: &str) -> Option<&'static EnvSpec> {
    AVAILABLE_ENVS.iter().find(|e| e.name == name)
}

pub fn get_env_training_params(name: &str) -> EnvTrainingParams {
    get_env_spec(name)
        .map(|s| s.default_params.clone())
        .unwrap_or(ENV_SOLO_V0_SPEC.default_params)
}

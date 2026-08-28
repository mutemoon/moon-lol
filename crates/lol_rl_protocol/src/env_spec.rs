use serde::{Deserialize, Serialize};

use crate::dsl::{EnvDslSpec, parse_env_dsl};

pub const ENV_SOLO_V0: &str = "SoloV0";
pub const ENV_FIORA_V3: &str = "FioraV3";
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

impl Default for EnvTrainingParams {
    fn default() -> Self {
        Self {
            lr: 3e-4,
            gamma: 0.99,
            gae_lambda: 0.95,
            clip_eps: 0.2,
            ppo_epochs: 8,
            hidden_dim: 64,
            rollout_steps_per_env: 160,
            total_iterations: 500,
        }
    }
}

/// 环境规范与展示元数据
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnvSpec {
    pub name: &'static str,
    pub label: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub num_agents: usize,
    pub default_params: EnvTrainingParams,
}

// ── 纯 .rl 规范文件载入（零硬编码 DSL 字符串） ────────────────────────────────
pub const ENV_SOLO_V0_DSL: &str = include_str!("../specs/solo_v0.rl");
pub const ENV_FIORA_V3_DSL: &str = include_str!("../specs/fiora_v3.rl");
pub const ENV_FIORA_V2_DSL: &str = include_str!("../specs/fiora_v2.rl");
pub const ENV_FIORA_V1_DSL: &str = include_str!("../specs/fiora_v1.rl");
pub const ENV_FIORA_V0_DSL: &str = include_str!("../specs/fiora_v0.rl");

/// 全局静态预解析的各环境 DSL 规范单例（零运行时解析开销）
pub static SPEC_SOLO_V0: std::sync::LazyLock<EnvDslSpec> = std::sync::LazyLock::new(|| {
    parse_env_dsl(ENV_SOLO_V0_DSL).expect("specs/solo_v0.rl DSL 规范脚本解析失败")
});

pub static SPEC_FIORA_V3: std::sync::LazyLock<EnvDslSpec> = std::sync::LazyLock::new(|| {
    parse_env_dsl(ENV_FIORA_V3_DSL).expect("specs/fiora_v3.rl DSL 规范脚本解析失败")
});

pub static SPEC_FIORA_V2: std::sync::LazyLock<EnvDslSpec> = std::sync::LazyLock::new(|| {
    parse_env_dsl(ENV_FIORA_V2_DSL).expect("specs/fiora_v2.rl DSL 规范脚本解析失败")
});

pub static SPEC_FIORA_V1: std::sync::LazyLock<EnvDslSpec> = std::sync::LazyLock::new(|| {
    parse_env_dsl(ENV_FIORA_V1_DSL).expect("specs/fiora_v1.rl DSL 规范脚本解析失败")
});

pub static SPEC_FIORA_V0: std::sync::LazyLock<EnvDslSpec> = std::sync::LazyLock::new(|| {
    parse_env_dsl(ENV_FIORA_V0_DSL).expect("specs/fiora_v0.rl DSL 规范脚本解析失败")
});

pub static ENV_SOLO_V0_SPEC: std::sync::LazyLock<EnvSpec> =
    std::sync::LazyLock::new(|| SPEC_SOLO_V0.to_env_spec());
pub static ENV_FIORA_V3_SPEC: std::sync::LazyLock<EnvSpec> =
    std::sync::LazyLock::new(|| SPEC_FIORA_V3.to_env_spec());
pub static ENV_FIORA_V2_SPEC: std::sync::LazyLock<EnvSpec> =
    std::sync::LazyLock::new(|| SPEC_FIORA_V2.to_env_spec());
pub static ENV_FIORA_V1_SPEC: std::sync::LazyLock<EnvSpec> =
    std::sync::LazyLock::new(|| SPEC_FIORA_V1.to_env_spec());
pub static ENV_FIORA_V0_SPEC: std::sync::LazyLock<EnvSpec> =
    std::sync::LazyLock::new(|| SPEC_FIORA_V0.to_env_spec());

pub static AVAILABLE_ENVS: std::sync::LazyLock<[&'static EnvSpec; 5]> =
    std::sync::LazyLock::new(|| {
        [
            &*ENV_SOLO_V0_SPEC,
            &*ENV_FIORA_V3_SPEC,
            &*ENV_FIORA_V2_SPEC,
            &*ENV_FIORA_V1_SPEC,
            &*ENV_FIORA_V0_SPEC,
        ]
    });

/// 获取环境对应的 DSL 源代码字符串
pub fn get_env_dsl_source(env_name: &str) -> Option<&'static str> {
    match env_name {
        ENV_SOLO_V0 => Some(ENV_SOLO_V0_DSL),
        ENV_FIORA_V3 => Some(ENV_FIORA_V3_DSL),
        ENV_FIORA_V2 => Some(ENV_FIORA_V2_DSL),
        ENV_FIORA_V1 => Some(ENV_FIORA_V1_DSL),
        ENV_FIORA_V0 => Some(ENV_FIORA_V0_DSL),
        _ => None,
    }
}

/// 获取环境对应的完整 DSL 规范对象引用（包含 ObsSchema, ActionSchema, RewardFormulaSpec）
pub fn get_env_dsl_spec(env_name: &str) -> Option<&'static EnvDslSpec> {
    match env_name {
        ENV_SOLO_V0 => Some(&SPEC_SOLO_V0),
        ENV_FIORA_V3 => Some(&SPEC_FIORA_V3),
        ENV_FIORA_V2 => Some(&SPEC_FIORA_V2),
        ENV_FIORA_V1 => Some(&SPEC_FIORA_V1),
        ENV_FIORA_V0 => Some(&SPEC_FIORA_V0),
        _ => None,
    }
}

/// 根据环境名称获取环境展示与超参数规范
pub fn get_env_spec(name: &str) -> Option<&'static EnvSpec> {
    match name {
        ENV_SOLO_V0 => Some(&ENV_SOLO_V0_SPEC),
        ENV_FIORA_V3 => Some(&ENV_FIORA_V3_SPEC),
        ENV_FIORA_V2 => Some(&ENV_FIORA_V2_SPEC),
        ENV_FIORA_V1 => Some(&ENV_FIORA_V1_SPEC),
        ENV_FIORA_V0 => Some(&ENV_FIORA_V0_SPEC),
        _ => None,
    }
}

/// 获取环境的默认训练超参数
pub fn get_env_training_params(name: &str) -> EnvTrainingParams {
    get_env_spec(name)
        .map(|s| s.default_params.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_available_envs_dsl_parse() {
        for &env in AVAILABLE_ENVS.iter() {
            let spec = get_env_dsl_spec(env.name)
                .unwrap_or_else(|| panic!("环境 {} 的 DSL 脚本应能成功解析", env.name));
            assert!(
                spec.obs_schema.is_some(),
                "环境 {} 应包含 ObsSchema",
                env.name
            );
            assert!(
                spec.action_schema.is_some(),
                "环境 {} 应包含 ActionSchema",
                env.name
            );
            assert!(
                spec.reward_formula.is_some(),
                "环境 {} 应包含 RewardFormulaSpec",
                env.name
            );
            assert_eq!(env.name, spec.name, "EnvSpec 名称与 DSL 解析名称一致");
        }
    }
}

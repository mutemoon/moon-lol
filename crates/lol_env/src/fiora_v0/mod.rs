pub mod action;
pub mod env;
pub mod step;

pub use action::*;
pub use env::*;
use lol_rl_protocol::ObsSchema;
pub use step::*;

pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FIORA_COMMON_OBS_SCHEMA, FioraVsRivenObs,
    VitalBreakTracker, compute_step_reward, get_obs_from_world, setup_skill_levels_world,
    unpause_virtual_time,
};

pub static FIORA_V0_SPEC: std::sync::LazyLock<&'static lol_rl_protocol::EnvDslSpec> =
    std::sync::LazyLock::new(|| &lol_rl_protocol::SPEC_FIORA_V0);

pub static FIORA_V0_OBS_SCHEMA: std::sync::LazyLock<ObsSchema> = std::sync::LazyLock::new(|| {
    FIORA_V0_SPEC
        .obs_schema
        .clone()
        .expect("SPEC_FIORA_V0 缺少 obs_schema")
});

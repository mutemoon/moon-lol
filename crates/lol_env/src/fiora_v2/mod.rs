pub mod action;
pub mod env;
pub mod obs;
pub mod reward;
pub mod step;

pub use action::*;
pub use env::*;
pub use obs::*;
pub use reward::*;
pub use step::*;

pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, AttackEventTracker, FioraRivenEntities, VitalBreakTracker,
    setup_skill_levels_world, unpause_virtual_time,
};
pub use crate::flash_plugin::{
    FLASH_COOLDOWN_SECS, FLASH_DISTANCE, FlashCooldown, dispatch_flash, extract_flash_obs,
    register_flash_plugin, tick_flash_cooldown,
};

pub static FIORA_V2_SPEC: std::sync::LazyLock<&'static lol_rl_protocol::EnvDslSpec> =
    std::sync::LazyLock::new(|| &lol_rl_protocol::SPEC_FIORA_V2);

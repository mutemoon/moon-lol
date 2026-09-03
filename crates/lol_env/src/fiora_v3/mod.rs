pub mod action;
pub mod env;
pub mod obs;
pub mod step;

pub use action::*;
pub use env::*;
pub use obs::*;
pub use step::*;

pub use crate::fiora_riven_common::{
    AttackEventTracker, setup_skill_levels_world, unpause_virtual_time,
};

pub static FIORA_V3_SPEC: std::sync::LazyLock<&'static lol_rl_protocol::EnvDslSpec> =
    std::sync::LazyLock::new(|| &lol_rl_protocol::SPEC_FIORA_V3);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::RlEnvironment;

    #[test]
    fn test_fiora_v3_obs_schema_and_dim() {
        let schema = FioraV3Env::obs_schema().expect("FioraV3 obs schema");
        assert_eq!(schema.raw_dim(), FioraV3Env::state_dim());
        assert_eq!(FioraV3Obs::dim(), FioraV3Env::state_dim());
        let labels = schema.to_dim_labels();
        assert_eq!(labels.len(), FioraV3Env::state_dim());
    }

    #[test]
    fn test_fiora_v3_action_schema() {
        let schema = FioraV3Env::action_schema().expect("FioraV3 action schema");
        assert_eq!(schema.encoding_dim(), 5); // 2 continuous + 1 action_type + 1 skill_slot + 1 unit selection
        assert_eq!(schema.num_branches(), 4);
        let labels = schema.to_encoding_labels();
        assert_eq!(labels.len(), 5);
    }

    #[test]
    fn test_fiora_v3_action_encoding_roundtrip() {
        let act = FioraV3Action::with_skill(
            0.5,
            -0.5,
            FioraV3DiscreteAction::CastSkill,
            FioraV3SkillSlot::W,
            3,
        );
        let encoded = act.to_encoding();
        assert_eq!(encoded.len(), 5);
        assert_eq!(encoded[0], 0.5);
        assert_eq!(encoded[1], -0.5);
        assert_eq!(encoded[2], 3.0); // discrete: CastSkill
        assert_eq!(encoded[3], 1.0); // skill_slot: W
        assert_eq!(encoded[4], 3.0); // target: 3

        let decoded = FioraV3Action::from_encoding(&encoded);
        assert_eq!(decoded.offset_x, 0.5);
        assert_eq!(decoded.offset_z, -0.5);
        assert_eq!(decoded.discrete, FioraV3DiscreteAction::CastSkill);
        assert_eq!(decoded.skill_slot, FioraV3SkillSlot::W);
        assert_eq!(decoded.target_idx, 3);
    }
}

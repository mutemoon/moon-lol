use lol_env::fiora_vs_riven::FioraVsRivenObs;
use lol_rl_protocol::ObsFeaturePayload;

/// Build an ObsFeaturePayload from a FioraVsRivenObs.
pub fn obs_feature_from_env(obs: &FioraVsRivenObs) -> ObsFeaturePayload {
    ObsFeaturePayload {
        fiora_hp_pct: if obs.fiora_max_hp > 0.0 {
            obs.fiora_hp / obs.fiora_max_hp
        } else {
            1.0
        },
        riven_hp_pct: if obs.riven_max_hp > 0.0 {
            obs.riven_hp / obs.riven_max_hp
        } else {
            1.0
        },
        distance: obs.distance,
        q_ready: obs.q_ready,
        w_ready: obs.w_ready,
        e_ready: obs.e_ready,
        r_ready: obs.r_ready,
        has_vital: obs.has_vital,
        vital_is_active: obs.vital_is_active,
        vital_direction: if obs.vital_dir_x > 0.5 {
            "+X (东侧)"
        } else if obs.vital_dir_neg_x > 0.5 {
            "-X (西侧)"
        } else if obs.vital_dir_z > 0.5 {
            "+Z (北侧)"
        } else if obs.vital_dir_neg_z > 0.5 {
            "-Z (南侧)"
        } else {
            "None"
        }
        .into(),
    }
}

pub mod driver;
pub mod models;
pub mod params;
pub mod rl;
pub mod systems;

use bevy::prelude::*;
pub use driver::*;
use lol_rpc::RpcAppExt;
pub use models::*;
pub use params::*;
pub use rl::*;
pub use systems::*;

pub struct PluginAgentObserver;

impl Plugin for PluginAgentObserver {
    fn build(&self, app: &mut App) {
        app.init_non_send::<driver::ScriptRuntimes>();
        app.init_resource::<rl::RlEnvs>();

        // 注册本模块提供的 RPC 命令
        app.register_rpc::<ObserveParams>("observe");
        app.register_rpc::<ActionParams>("action");
        app.register_rpc::<SetScriptParams>("set_script");
        app.register_rpc::<RlResetParams>("rl_reset");
        app.register_rpc::<RlStepParams>("rl_step");
        app.register_rpc::<GetAgentsParams>("get_agents");

        app.add_observer(on_observe)
            .add_observer(on_action)
            .add_observer(on_set_script)
            .add_observer(on_rl_reset)
            .add_observer(on_rl_step)
            .add_observer(on_get_agents);
        app.add_systems(FixedUpdate, drive_script_agents);
    }
}

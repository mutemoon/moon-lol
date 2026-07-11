use bevy::app::Plugin;

// 位移框架组合化重构期间，仅编译上单四姐妹（riven/fiora/camille/irelia）
// 与五虎中已实现的英雄（darius/aatrox/sett/volibear；铁男 mordekaiser 尚未实现）。
// 其余英雄源码保留在磁盘上但暂不参与编译，待重构完成后恢复。

pub mod aatrox;
pub mod camille;
pub mod darius;
pub mod fiora;
pub mod irelia;
pub mod riven;
pub mod sett;
pub mod volibear;

#[cfg(test)]
mod test_utils;

#[derive(Default)]
pub struct PluginChampions;

impl Plugin for PluginChampions {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(aatrox::PluginAatrox);
        app.add_plugins(camille::PluginCamille);
        app.add_plugins(darius::PluginDarius);
        app.add_plugins(fiora::PluginFiora);
        app.add_plugins(irelia::PluginIrelia);
        app.add_plugins(riven::PluginRiven);
        app.add_plugins(sett::PluginSett);
        app.add_plugins(volibear::PluginVolibear);
    }
}

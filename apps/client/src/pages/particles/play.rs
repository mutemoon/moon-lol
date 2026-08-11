//! 播放 / 重播：序列化工作副本 → 粒子 WS 播放，含单个发射器与重置。

use gpui::*;

use super::edit::{primary_list_mut, primary_list_ref};
use super::input::clear_all_input_buffers;
use crate::components::sidebar::AppSidebar;
use crate::services::particle_service;

pub(super) fn spawn_play_ron(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>, ron: String) {
    let handle = sidebar.particles.ws_handle.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                if let Some(h) = handle {
                    if let Err(e) = h.play_particle(&ron).await {
                        this.update(&mut cx, |this, cx| {
                            this.particles.error = Some(e);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }
        },
    )
    .detach();
}

/// 序列化工作副本并播放（改动后自动重播的落地点）。
pub(super) fn play_working(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let ron = sidebar
        .particles
        .working_def
        .as_ref()
        .map(particle_service::serialize_vfx_system);
    match ron {
        Some(Ok(r)) => spawn_play_ron(sidebar, cx, r),
        Some(Err(e)) => {
            sidebar.particles.error = Some(e);
            cx.notify();
        }
        None => {}
    }
}

/// 编辑提交后：若开启「改动后自动播放」则重播。
pub(super) fn replay_after_edit(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    if sidebar.particles.auto_play {
        play_working(sidebar, cx);
    }
    cx.notify();
}

pub(super) fn stop_playing(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let handle = sidebar.particles.ws_handle.clone();
    if let Some(h) = handle {
        cx.spawn(
            move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let this = this.clone();
                let mut cx = cx.clone();
                async move {
                    if let Err(e) = h.stop_particle().await {
                        this.update(&mut cx, |this, cx| {
                            this.particles.error = Some(e);
                            cx.notify();
                        })
                        .ok();
                    }
                }
            },
        )
        .detach();
    } else {
        sidebar.particles.error = Some("粒子 server 未连接".to_string());
        cx.notify();
    }
}

/// 仅播放单个发射器（保留其所在列表，其余清空）。
pub(super) fn play_single_emitter(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    idx: usize,
) {
    let ron = (|| -> Option<String> {
        let wd = sidebar.particles.working_def.as_ref()?;
        let mut single = wd.clone();
        let (use_complex, em) = {
            let c = single
                .complex_emitter_definition_data
                .as_ref()
                .and_then(|l| l.get(idx).cloned());
            let c2 = single
                .simple_emitter_definition_data
                .as_ref()
                .and_then(|l| l.get(idx).cloned());
            if c.is_some() {
                (true, c)
            } else {
                (false, c2)
            }
        };
        let em = em?;
        if use_complex {
            single.complex_emitter_definition_data = Some(vec![em]);
            single.simple_emitter_definition_data = None;
        } else {
            single.simple_emitter_definition_data = Some(vec![em]);
            single.complex_emitter_definition_data = None;
        }
        particle_service::serialize_vfx_system(&single).ok()
    })();
    if let Some(r) = ron {
        spawn_play_ron(sidebar, cx, r);
    }
}

/// 重置单个发射器为初始备份值。
pub(super) fn reset_single_emitter(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    idx: usize,
) {
    let changed = (|| -> Option<()> {
        let backup = sidebar.particles.initial_def_backup.as_ref()?;
        let wd = sidebar.particles.working_def.as_mut()?;
        let backup_em = primary_list_ref(backup)?.get(idx).cloned()?;
        let list = primary_list_mut(wd)?;
        if idx < list.len() {
            list[idx] = backup_em;
            Some(())
        } else {
            None
        }
    })();
    if changed.is_some() {
        clear_all_input_buffers();
        replay_after_edit(sidebar, cx);
    }
}

/// 重置整个系统为初始备份定义并重播。
pub(super) fn reset_system(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    if let Some(b) = &sidebar.particles.initial_def_backup {
        sidebar.particles.working_def = Some(b.clone());
    }
    clear_all_input_buffers();
    play_working(sidebar, cx);
}

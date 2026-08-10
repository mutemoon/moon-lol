//! 发射器编辑器：基本参数 / 渲染标志 / 采样器 / 贴图资源。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_share::ConfigVfxEmitterDefinition;

use super::edit::{
    get_flag, get_num_field, get_texture, read_sampler, set_name_idx, set_num_field,
    set_sampler_component, set_tex_div_comp, set_texture_idx, tex_div_values, NumField,
    SamplerKind, FLAGS, TEX_ITEMS,
};
use super::input::{
    render_flag_toggle, render_number_input, render_sampler_mode_dropdown, render_text_input,
};
use super::play::{play_single_emitter, reset_single_emitter};
use crate::components::sidebar::AppSidebar;

fn comp_labels(dims: usize) -> &'static [&'static str] {
    match dims {
        2 => &["X", "Y"],
        4 => &["R", "G", "B", "A"],
        _ => &["X", "Y", "Z"],
    }
}

fn render_section(
    cx: &mut Context<AppSidebar>,
    title: &str,
    children: Vec<AnyElement>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .border_b_1()
                .border_color(cx.theme().border.opacity(0.6))
                .pb_1()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().accent)
                .child(title.to_string()),
        )
        .children(children)
        .into_any_element()
}

fn render_sampler_row(
    cx: &mut Context<AppSidebar>,
    hash: u32,
    idx: usize,
    kind: SamplerKind,
    em: &ConfigVfxEmitterDefinition,
) -> AnyElement {
    let (vals, is_curve) = read_sampler(em, kind);
    let dims = kind.dims();
    let labels = comp_labels(dims);
    let muted = cx.theme().muted_foreground;

    v_flex()
        .gap_1()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border.opacity(0.4))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().font_bold().child(kind.label()))
                .child(render_sampler_mode_dropdown(
                    cx,
                    format!("{:08x}-{}-sm-{:?}-mode", hash, idx, kind),
                    idx,
                    kind,
                    is_curve,
                )),
        )
        .child(h_flex().gap_1().children((0..dims).map(|c| {
            let id = format!("{:08x}-{}-sm-{:?}-{}", hash, idx, kind, c);
            v_flex()
                .gap_0p5()
                .flex_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(labels.get(c).copied().unwrap_or("?").to_string()),
                )
                .child(render_number_input(
                    cx,
                    id,
                    vals.get(c).copied().unwrap_or(0.0),
                    move |v| set_sampler_component(idx, kind, c, v),
                ))
                .into_any_element()
        })))
        .into_any_element()
}

pub(super) fn render_emitter_editor(
    cx: &mut Context<AppSidebar>,
    hash: u32,
    idx: usize,
    em: &ConfigVfxEmitterDefinition,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let title = em
        .emitter_name
        .clone()
        .unwrap_or_else(|| format!("发射器 #{}", idx + 1));

    // 基本参数
    let basic_fields = vec![
        (
            "名称 emitter_name".to_string(),
            render_text_input(
                cx,
                format!("{:08x}-{}-name", hash, idx),
                em.emitter_name.clone().unwrap_or_default(),
                "Fire_Particle",
                move |v| set_name_idx(idx, v),
            ),
        ),
        (
            "寿命 lifetime".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-lifetime", hash, idx),
                get_num_field(em, NumField::Lifetime),
                move |v| set_num_field(idx, NumField::Lifetime, v),
            ),
        ),
        (
            "帧数 num_frames".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-num_frames", hash, idx),
                get_num_field(em, NumField::NumFrames),
                move |v| set_num_field(idx, NumField::NumFrames, v),
            ),
        ),
        (
            "混合模式 blend_mode".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-blend", hash, idx),
                get_num_field(em, NumField::BlendMode),
                move |v| set_num_field(idx, NumField::BlendMode, v),
            ),
        ),
        (
            "Alpha参考 alpha_ref".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-alpha", hash, idx),
                get_num_field(em, NumField::AlphaRef),
                move |v| set_num_field(idx, NumField::AlphaRef, v),
            ),
        ),
    ];

    // 贴图
    let mut texture_children: Vec<AnyElement> = TEX_ITEMS
        .iter()
        .map(|(f, label, placeholder)| {
            let f = *f;
            v_flex()
                .gap_1()
                .flex_1()
                .child(div().text_xs().text_color(muted).child(label.to_string()))
                .child(render_text_input(
                    cx,
                    format!("{:08x}-{}-tex-{:?}", hash, idx, f),
                    get_texture(em, f),
                    placeholder,
                    move |v| set_texture_idx(idx, f, v),
                ))
                .into_any_element()
        })
        .collect();
    // tex_div
    let tv = tex_div_values(em);
    texture_children.push(
        v_flex()
            .gap_1()
            .flex_1()
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("贴图分割 tex_div (U/V)".to_string()),
            )
            .child(h_flex().gap_1().children((0..2).map(|c| {
                let id = format!("{:08x}-{}-texdiv-{}", hash, idx, c);
                render_number_input(cx, id, tv[c], move |v| set_tex_div_comp(idx, c, v))
                    .into_any_element()
            })))
            .into_any_element(),
    );

    let flag_children = vec![h_flex()
        .gap_3()
        .flex_wrap()
        .children(FLAGS.iter().map(|(flag, label)| {
            let checked = get_flag(em, *flag);
            render_flag_toggle(
                cx,
                format!("{:08x}-{}-flag-{:?}", hash, idx, flag),
                idx,
                *flag,
                label,
                checked,
            )
        }))
        .into_any_element()];
    let sampler_children: Vec<AnyElement> = SamplerKind::all()
        .iter()
        .map(|k| render_sampler_row(cx, hash, idx, *k, em))
        .collect();

    v_flex()
        .gap_4()
        // 发射器工具条
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Settings)
                        .child(div().font_bold().text_sm().child(title)),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new(format!("play-single-{:08x}-{}", hash, idx))
                                .icon(IconName::Play)
                                .label("播放单个")
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    play_single_emitter(cx, idx);
                                })),
                        )
                        .child(
                            Button::new(format!("reset-single-{:08x}-{}", hash, idx))
                                .ghost()
                                .label("重置")
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    reset_single_emitter(cx, idx);
                                })),
                        ),
                ),
        )
        // 基本参数
        .child(render_section(
            cx,
            "基本参数",
            vec![h_flex()
                .gap_3()
                .flex_wrap()
                .children(basic_fields.into_iter().map(|(label, input)| {
                    v_flex()
                        .gap_1()
                        .w_40()
                        .child(div().text_xs().text_color(muted).child(label))
                        .child(input)
                        .into_any_element()
                }))
                .into_any_element()],
        ))
        // 渲染标志
        .child(render_section(cx, "渲染标志", flag_children))
        // 采样器
        .child(render_section(
            cx,
            "采样器（数值输入 + 常量/曲线预设）",
            sampler_children,
        ))
        // 贴图资源
        .child(render_section(cx, "贴图资源", texture_children))
        .into_any_element()
}

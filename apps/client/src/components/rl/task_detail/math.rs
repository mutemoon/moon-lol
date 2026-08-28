use gpui::*;
use gpui_component::ActiveTheme;
use ratex_layout::{layout, to_display_list, LayoutOptions};
use ratex_parser::parser::parse;
use ratex_svg::{render_to_svg, SvgOptions};
use ratex_types::math_style::MathStyle;

use crate::components::sidebar::AppSidebar;

/// 公式在面板里的基准行高（px），与 text_sm/text_xs 视觉协调。
const FORMULA_HEIGHT: f32 = 22.0;

/// 把 LaTeX 源码渲染成单行公式；解析/排版失败时回退为纯文本。
pub fn render_math(latex: &str, cx: &Context<AppSidebar>) -> AnyElement {
    let Some((svg, aspect)) = latex_to_svg(latex) else {
        return div().text_sm().child(latex.to_string()).into_any_element();
    };

    let color = cx.theme().foreground;
    let height = FORMULA_HEIGHT;
    let width = (height * aspect).max(4.0);
    let key: SharedString = latex.into();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let _ = window.paint_svg(
                bounds,
                key,
                Some(svg.as_bytes()),
                TransformationMatrix::unit(),
                color,
                cx,
            );
        },
    )
    .w(px(width))
    .h(px(height))
    .into_any_element()
}

/// 解析 + 排版 + 生成自包含 SVG，返回 (svg 字符串, 宽高比)。
fn latex_to_svg(latex: &str) -> Option<(String, f32)> {
    let nodes = parse(latex).ok()?;
    let opts = LayoutOptions::default().with_style(MathStyle::Text);
    let root = layout(&nodes, &opts);
    let list = to_display_list(&root);
    let svg_opts = SvgOptions {
        font_size: 40.0,
        padding: 2.0,
        embed_glyphs: true,
        ..SvgOptions::default()
    };
    let aspect = (list.width * svg_opts.font_size + 2.0 * svg_opts.padding) as f32
        / ((list.height + list.depth) * svg_opts.font_size + 2.0 * svg_opts.padding) as f32;
    let svg = render_to_svg(&list, &svg_opts);
    Some((svg, aspect))
}

#[cfg(test)]
mod tests {
    use lol_rl_protocol::{RewardExpr, RewardFormulaSpec, RewardTermSpec};

    use super::latex_to_svg;

    fn fiora_formula() -> RewardFormulaSpec {
        RewardFormulaSpec {
            name: "test".into(),
            terms: vec![
                RewardTermSpec::new("time", "时间", RewardExpr::Constant(-0.002)),
                RewardTermSpec::new(
                    "align",
                    "对齐",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.02)),
                        Box::new(RewardExpr::Variable("is_newly_aligned".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "vital",
                    "破绽",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(0.8)),
                        Box::new(RewardExpr::Variable("is_vital_break".into())),
                    ),
                ),
            ],
        }
    }

    #[test]
    fn latex_to_svg_emits_path_glyphs() {
        let latex = fiora_formula().to_latex();
        let (svg, aspect) = latex_to_svg(&latex).expect("parse ok");
        assert!(aspect > 0.0, "aspect={aspect}, latex={latex}");
        assert!(
            svg.contains("<path"),
            "no <path> glyphs, latex={latex}:\n{svg}"
        );
    }
}

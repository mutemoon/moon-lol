use gpui::*;
use rust_i18n::t;

use crate::types::ActiveView;

pub fn render_placeholder(active: ActiveView) -> AnyElement {
    div()
        .p_4()
        .child(t!(
            "app.placeholder.current_view",
            view = format!("{:?}", active)
        ))
        .into_any_element()
}

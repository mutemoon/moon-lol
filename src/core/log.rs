use bevy::log::tracing_subscriber::fmt;

/// 创建自定义的 Bevy LogPlugin，包含源码位置
pub fn create_log_plugin() -> bevy::log::LogPlugin {
    bevy::log::LogPlugin {
        fmt_layer: |_app| {
            Some(Box::new(
                fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_writer(std::io::stdout),
            ))
        },
        ..Default::default()
    }
}

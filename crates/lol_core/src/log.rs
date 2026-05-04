use std::io;
use std::sync::Mutex;

use bevy::log::tracing_subscriber::fmt;
use bevy::prelude::*;

/// Strip ANSI escape sequences (e.g. `\x1b[32m`, `\x1b[0m`) from a string.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.as_str().starts_with('[') {
            // Skip the '[' and all characters until 'm'
            chars.next(); // skip '['
            for ch in chars.by_ref() {
                if ch == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Resource holding the receive-half of the log bridge.
/// `PluginDebugPanel` drains this and forwards entries via WebSocket.
#[derive(Resource)]
pub struct LogReceiver(pub async_channel::Receiver<String>);

/// Global channel for forwarding formatted log lines to the WebSocket debug panel.
static LOG_BRIDGE_TX: Mutex<Option<async_channel::Sender<String>>> = Mutex::new(None);

/// Writer that duplicates output to stdout and the WS log bridge channel.
struct BridgeWriter {
    buffer: Vec<u8>,
}

impl io::Write for BridgeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        io::stdout().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

impl Drop for BridgeWriter {
    fn drop(&mut self) {
        if let Ok(s) = String::from_utf8(std::mem::take(&mut self.buffer)) {
            let s = strip_ansi(s.trim_end());
            if !s.is_empty() {
                if let Some(tx) = LOG_BRIDGE_TX.lock().unwrap().as_ref() {
                    let _ = tx.try_send(s);
                }
            }
        }
    }
}

struct BridgeMakeWriter;

impl<'a> fmt::MakeWriter<'a> for BridgeMakeWriter {
    type Writer = BridgeWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BridgeWriter { buffer: Vec::new() }
    }
}

/// Create a custom Bevy LogPlugin with source location info.
/// Returns the LogPlugin and a receiver that receives formatted log lines
/// for forwarding to the WebSocket debug panel.
pub fn create_log_plugin() -> (bevy::log::LogPlugin, async_channel::Receiver<String>) {
    let (tx, rx) = async_channel::unbounded::<String>();
    *LOG_BRIDGE_TX.lock().unwrap() = Some(tx);

    let plugin = bevy::log::LogPlugin {
        filter: "bevy_gltf_draco=off".to_owned(),
        fmt_layer: |_app| {
            Some(Box::new(
                fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_writer(BridgeMakeWriter),
            ))
        },
        ..Default::default()
    };

    (plugin, rx)
}

use std::path::PathBuf;
use std::process::Child;

use crate::ws::WsSession;

/// Global application state.
pub struct AppState {
    /// Running Bevy process, if any.
    pub bevy: Option<BevyProcess>,
    /// Active WebSocket session, if any.
    pub ws: Option<WsSession>,
}

pub struct BevyProcess {
    pub child: Child,
    pub port: u16,
    pub log_db_path: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            bevy: None,
            ws: None,
        }
    }
}


use serde::{Deserialize, Serialize};

/// 队伍数据
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamData {
    pub team: u32,
}

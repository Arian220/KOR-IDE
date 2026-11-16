use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PortInfo {
    pub address: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub protocol_label: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardCandidate {
    pub name: String,
    pub fqbn: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardInfo {
    pub port: PortInfo,
    #[serde(default)]
    pub matching_boards: Vec<BoardCandidate>,
    pub protocol: Option<String>,
    pub protocol_label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BoardListResponse {
    #[serde(default)]
    pub detected_ports: Option<Vec<DetectedPort>>,
}

#[derive(Debug, Deserialize)]
pub struct DetectedPort {
    pub port: PortInfo,
    #[serde(default)]
    pub matching_boards: Option<Vec<BoardCandidate>>,
}

impl BoardListResponse {
    pub fn into_board_infos(self) -> Vec<BoardInfo> {
        self.detected_ports
            .unwrap_or_default()
            .into_iter()
            .map(|detected| BoardInfo {
                protocol: detected.port.protocol.clone(),
                protocol_label: detected.port.protocol_label.clone(),
                port: detected.port,
                matching_boards: detected.matching_boards.unwrap_or_default(),
            })
            .collect()
    }
}

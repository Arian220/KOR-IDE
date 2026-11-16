use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sketch {
    pub id: String,
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub fqbn: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SketchSummary {
    pub id: String,
    pub name: String,
    pub updated_at: String,
    #[serde(default)]
    pub fqbn: Option<String>,
}

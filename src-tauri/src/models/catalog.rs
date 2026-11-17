use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CoreInfo {
    pub id: String,
    pub installed: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryInfo {
    pub name: String,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub author: Option<String>,
    pub sentence: Option<String>,
}

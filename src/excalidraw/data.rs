use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Selection,
    Hand,
    Rectangle,
    Ellipse,
    Diamond,
    Line,
    Arrow,
    Freedraw,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExcalidrawRoundness {
    #[serde(rename = "type")]
    pub round_type: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcalidrawFile {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub id: String,
    #[serde(rename = "dataURL")]
    pub data_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcalidrawElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(rename = "strokeColor")]
    pub stroke_color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    #[serde(rename = "fillStyle")]
    pub fill_style: String,
    #[serde(rename = "strokeWidth")]
    pub stroke_width: i32,
    #[serde(rename = "strokeStyle")]
    pub stroke_style: String,
    pub opacity: i32,
    pub roughness: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<[f32; 2]>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, rename = "rawText", skip_serializing_if = "Option::is_none")]
    pub raw_text: Option<String>,
    #[serde(default, rename = "originalText", skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, rename = "fontSize", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roundness: Option<ExcalidrawRoundness>,
    #[serde(default, rename = "endArrowhead", skip_serializing_if = "Option::is_none")]
    pub end_arrowhead: Option<String>,
    #[serde(default, rename = "boundElements", skip_serializing_if = "Option::is_none")]
    pub bound_elements: Option<Vec<BoundElement>>,
    #[serde(default, rename = "containerId", skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,
    #[serde(default, rename = "fileId", skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    pub seed: i32,
    pub version: i32,
    #[serde(rename = "versionNonce")]
    pub version_nonce: i32,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
    #[serde(rename = "groupIds")]
    pub group_ids: Vec<String>,
    #[serde(default, rename = "frameId", skip_serializing_if = "Option::is_none")]
    pub frame_id: Option<String>,
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
}

impl Default for ExcalidrawElement {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            element_type: "rectangle".to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            angle: 0.0,
            stroke_color: "#000000".to_string(),
            background_color: "transparent".to_string(),
            fill_style: "solid".to_string(),
            stroke_width: 2,
            stroke_style: "solid".to_string(),
            opacity: 100,
            roughness: 1,
            points: vec![],
            text: "".to_string(),
            raw_text: None,
            original_text: None,
            font_size: Some(20.0),
            roundness: Some(ExcalidrawRoundness { round_type: 3 }),
            end_arrowhead: None,
            bound_elements: None,
            container_id: None,
            file_id: None,
            link: None,
            scale: None,
            seed: 0,
            version: 1,
            version_nonce: 0,
            is_deleted: false,
            group_ids: vec![],
            frame_id: None,
            locked: false,
            updated: None,
            index: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcalidrawAppState {
    #[serde(rename = "viewBackgroundColor")]
    pub view_background_color: String,
    #[serde(default, rename = "showGrid")]
    pub show_grid: bool,
    #[serde(default, rename = "snapEnabled")]
    pub snap_enabled: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for ExcalidrawAppState {
    fn default() -> Self {
        Self {
            view_background_color: "#ffffff".to_string(),
            show_grid: true,
            snap_enabled: true,
            extra: serde_json::Map::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcalidrawScene {
    #[serde(default, rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub source: String,
    pub elements: Vec<ExcalidrawElement>,
    #[serde(default, rename = "appState")]
    pub app_state: ExcalidrawAppState,
    #[serde(default)]
    pub files: HashMap<String, ExcalidrawFile>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for ExcalidrawScene {
    fn default() -> Self {
        Self {
            type_: "excalidraw".to_string(),
            version: 2,
            source: "https://excalidraw.com".to_string(),
            elements: vec![],
            app_state: ExcalidrawAppState::default(),
            files: HashMap::new(),
            extra: serde_json::Map::new(),
        }
    }
}

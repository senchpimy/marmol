use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Selection,
    Rectangle,
    Ellipse,
    Diamond,
    Line,
    Arrow,
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
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub angle: f32,
    #[serde(default, rename = "strokeColor")]
    pub stroke_color: String,
    #[serde(default, rename = "backgroundColor")]
    pub background_color: String,
    #[serde(default, rename = "fillStyle")]
    pub fill_style: String,
    #[serde(default, rename = "strokeWidth")]
    pub stroke_width: f32,
    #[serde(default, rename = "strokeStyle")]
    pub stroke_style: String,
    #[serde(default)]
    pub opacity: f32,
    #[serde(default)]
    pub roughness: f32,
    #[serde(default)]
    pub points: Vec<[f32; 2]>,
    #[serde(default)]
    pub text: String,
    #[serde(default, rename = "fontSize")]
    pub font_size: f32,
    #[serde(default)]
    pub roundness: Option<ExcalidrawRoundness>,
    #[serde(default, rename = "endArrowhead")]
    pub end_arrowhead: Option<String>,
    #[serde(default, rename = "boundElements")]
    pub bound_elements: Option<Vec<BoundElement>>,
    #[serde(default, rename = "containerId")]
    pub container_id: Option<String>,
    #[serde(default, rename = "fileId")]
    pub file_id: Option<String>,
    #[serde(default)]
    pub scale: Option<[f32; 2]>,
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
            stroke_color: "#BBBBBB".to_string(),
            background_color: "transparent".to_string(),
            fill_style: "solid".to_string(),
            stroke_width: 1.0,
            stroke_style: "solid".to_string(),
            opacity: 100.0,
            roughness: 1.0,
            points: vec![],
            text: "".to_string(),
            font_size: 20.0,
            roundness: Some(ExcalidrawRoundness { round_type: 3 }),
            end_arrowhead: None,
            bound_elements: None,
            container_id: None,
            file_id: None,
            scale: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExcalidrawScene {
    #[serde(default)]
    pub type_: String,
    #[serde(default)]
    pub version: i32,
    #[serde(default)]
    pub source: String,
    pub elements: Vec<ExcalidrawElement>,
    #[serde(default)]
    pub files: HashMap<String, ExcalidrawFile>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_URL: &str = "";
pub const DEFAULT_API_KEY: &str = "";
pub const DEFAULT_MODEL: &str = "";

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigLlm {
    pub url: String,
    pub api_key: String,
    pub modelo: String,
}

impl Default for ConfigLlm {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.to_string(),
            api_key: DEFAULT_API_KEY.to_string(),
            modelo: DEFAULT_MODEL.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct SketchConfig {
    #[serde(default)]
    pub llm: ConfigLlm,
}

fn ruta() -> PathBuf {
    let dir = crate::configuraciones::get_config_dir();
    PathBuf::from(dir).join("sketch.json")
}

pub fn cargar() -> SketchConfig {
    match std::fs::read_to_string(ruta()) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => SketchConfig::default(),
    }
}

pub fn guardar(cfg: &SketchConfig) {
    if let Ok(data) = serde_json::to_string_pretty(cfg) {
        let dir = crate::configuraciones::get_config_dir();
        std::fs::create_dir_all(&dir).ok();
        let _ = std::fs::write(ruta(), data);
    }
}

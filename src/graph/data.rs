use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MarmolPoint {
    pub text: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub rel_path: String,
    pub abs_path: String,
    pub is_attachment: bool,
    pub is_tag: bool,
    pub exists: bool,
}

impl MarmolPoint {
    pub fn new(
        val: &str,
        tags: Vec<String>,
        links: Vec<String>,
        rel_path: String,
        abs_path: String,
        is_attachment: bool,
        is_tag: bool,
        exists: bool,
    ) -> Self {
        Self {
            text: format!("{}", val),
            tags,
            links,
            rel_path,
            abs_path,
            is_attachment,
            is_tag,
            exists,
        }
    }
}

/// Tipo de coincidencia de un término de búsqueda. Replica los operadores de
/// la búsqueda de Obsidian (`file:`, `path:`, `tag:`, `line:`, `section:`,
/// `content:`) y el término libre (`Any`).
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum MatchType {
    Any,
    Filename,
    Tag,
    Path,
    Content,
    Section,
}

impl Default for MatchType {
    fn default() -> Self {
        MatchType::Any
    }
}

/// Un grupo de color. Al igual que en Obsidian se define mediante una consulta
/// (que admite los mismos operadores que la búsqueda) y un color.
#[derive(Clone, Serialize, Deserialize)]
pub struct CustomGroup {
    pub query: String,
    pub color: Color32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl CustomGroup {
    pub fn new(query: String, color: Color32) -> Self {
        Self {
            query,
            color,
            enabled: true,
        }
    }
}

/// Estado serializable del grafo para persistirlo por vault (como el
/// `.obsidian/graph.json` de Obsidian).
#[derive(Serialize, Deserialize)]
pub struct GraphSettings {
    pub center_force: f32,
    pub repel_force: f32,
    pub link_force: f32,
    pub link_distance: f32,
    #[serde(default)]
    pub filter_query: String,
    #[serde(default = "default_true")]
    pub show_attachments: bool,
    #[serde(default)]
    pub show_existing_only: bool,
    #[serde(default = "default_true")]
    pub show_orphans: bool,
    #[serde(default)]
    pub show_tags: bool,
    #[serde(default)]
    pub show_arrows: bool,
    #[serde(default = "default_text_threshold")]
    pub text_zoom_threshold: f64,
    #[serde(default = "default_node_size")]
    pub node_size: f32,
    #[serde(default = "default_line_thickness")]
    pub line_thickness: f32,
    #[serde(default)]
    pub custom_groups: Vec<CustomGroup>,
}

fn default_text_threshold() -> f64 {
    500.0
}

fn default_node_size() -> f32 {
    7.0
}

fn default_line_thickness() -> f32 {
    2.0
}

pub struct Graph {
    // Datos principales (Hacemos pub para que la UI pueda leerlos)
    pub points: Vec<MarmolPoint>,
    pub points_coord: Vec<(f32, f32)>,
    pub velocities: Vec<Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub node_degrees: Vec<usize>,

    // Fuerzas Físicas (pub para los sliders)
    pub center_force: f32,
    pub repel_force: f32,
    pub link_force: f32,
    /// Distancia de reposo de los enlaces (equivalente a "Link distance" de
    /// Obsidian). 0 = longitud automática.
    pub link_distance: f32,
    pub group_force: f32,
    pub tag_force: f32,

    // Filtro de búsqueda (misma sintaxis que Obsidian)
    pub filter_query: String,

    // Toggles
    pub show_attachments: bool,
    pub show_existing_only: bool,
    pub show_orphans: bool,
    pub show_tags: bool,

    // Visualización
    pub show_arrows: bool,
    pub text_zoom_threshold: f64,
    pub node_size: f32,
    pub line_thickness: f32,

    // Estado interno UI (Drag & Drop / interacción)
    pub dragged_node_index: Option<usize>,
    pub hovered_node_index: Option<usize>,
    pub selected_node_index: Option<usize>,
    pub show_controls: bool,
    pub pending_fit: bool,

    // Colores
    pub orphan_color: Color32,
    pub ghost_color: Color32,
    pub attachment_color: Color32,
    pub tag_node_color: Color32,
    pub palette: Vec<Color32>,
    pub tags_colors: HashMap<String, Color32>,

    // Grupos personalizados (UI State)
    pub custom_groups: Vec<CustomGroup>,
    pub new_group_query: String,
    pub new_group_col: Color32,

    // Caché del contenido de los archivos (abs_path -> contenido) para que los
    // filtros Content/Section no lean disco en el hot path de renderizado/física.
    pub content_cache: HashMap<String, String>,

    // Ruta del vault, para persistir los ajustes del grafo.
    pub vault_path: String,

    // Marca que hay ajustes pendientes de guardar (se guardan al soltar el
    // puntero para no escribir en disco en cada frame de un slider).
    pub settings_dirty: bool,

    // Indica si ya se indexó el vault. Permite cargar los datos de forma
    // perezosa (solo al mostrar el grafo) en vez de leer todo el vault al iniciar.
    pub loaded: bool,
}

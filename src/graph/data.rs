use egui::{Color32, Vec2};
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

#[derive(Clone, PartialEq)]
pub enum MatchType {
    Filename,
    Tag,
    Path,
    Content,
    Section,
}

#[derive(Clone)]
pub struct CustomGroup {
    pub match_type: MatchType,
    pub value: String,
    pub color: Color32,
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
    pub group_force: f32,
    pub tag_force: f32,

    // Filtros de Texto
    pub filter_filename: String,
    pub filter_tag: String,
    pub filter_path: String,
    pub filter_line: String,
    pub filter_section: String,

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

    // Estado interno UI (Drag & Drop)
    pub dragged_node_index: Option<usize>,

    // Colores
    pub orphan_color: Color32,
    pub ghost_color: Color32,
    pub attachment_color: Color32,
    pub palette: Vec<Color32>,
    pub tags_colors: HashMap<String, Color32>,

    // Grupos personalizados (UI State)
    pub custom_groups: Vec<CustomGroup>,
    pub new_group_type: MatchType,
    pub new_group_val: String,
    pub new_group_col: Color32,
    pub hovered_node_index: Option<usize>,
}

use directories::BaseDirs;
use egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, TextStyle,
    Visuals,
};
use font_loader::system_fonts::{self, FontPropertyBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct ThemeConfig {
    bg_window: Option<String>,
    bg_panel: Option<String>,
    bg_input: Option<String>,
    bg_faint: Option<String>,

    text_primary: Option<String>,
    text_secondary: Option<String>,

    accent: Option<String>,
    border: Option<String>,

    success: Option<String>,
    error: Option<String>,
    warn: Option<String>,

    font_family: Option<String>,
    base_font_size: Option<f32>,
    corner_radius: Option<u8>,
}

pub fn load_and_apply_theme(ctx: &Context) {
    let binding = BaseDirs::new().unwrap();
    let config_dir = binding.config_dir();
    let theme_path = config_dir.join("marmol").join("theme.json");

    println!("Intentando cargar tema desde: {:?}", theme_path);

    if !theme_path.exists() {
        println!("No existe theme.json. Usando tema por defecto de la librería.");
        return;
    }

    let theme_config: ThemeConfig = match fs::read_to_string(&theme_path) {
        Ok(data) => {
            println!("Archivo encontrado. Parseando...");
            match serde_json::from_str(&data) {
                Ok(cfg) => {
                    println!("Tema cargado exitosamente.");
                    cfg
                }
                Err(e) => {
                    eprintln!("Error de sintaxis en theme.json: {}. Se usarán valores por defecto de la librería para los campos ilegibles.", e);
                    ThemeConfig::default()
                }
            }
        }
        Err(e) => {
            eprintln!(
                "Error leyendo archivo: {}. Usando tema por defecto de la librería.",
                e
            );
            return;
        }
    };

    apply_visuals(ctx, &theme_config);

    if let Some(font_name) = &theme_config.font_family {
        load_system_font(ctx, font_name);
    }

    if let Some(size) = theme_config.base_font_size {
        setup_font_sizes(ctx, size);
    }
}

fn apply_visuals(ctx: &Context, t: &ThemeConfig) {
    let mut v = Visuals::dark();

    let to_col = |opt: &Option<String>| opt.as_deref().map(hex_to_color);

    if let Some(c) = to_col(&t.bg_window) {
        v.window_fill = c;
    }
    if let Some(c) = to_col(&t.bg_panel) {
        v.panel_fill = c;
    }
    if let Some(c) = to_col(&t.bg_faint) {
        v.faint_bg_color = c;
    }
    if let Some(c) = to_col(&t.bg_input) {
        v.extreme_bg_color = c;
        v.code_bg_color = c;
        v.text_edit_bg_color = Some(c);
    }

    if let Some(c) = to_col(&t.text_primary) {
        v.override_text_color = Some(c);
        v.widgets.noninteractive.fg_stroke.color = c;
        v.widgets.inactive.fg_stroke.color = c;
        v.widgets.open.fg_stroke.color = c;
    }

    if let Some(c) = to_col(&t.accent) {
        v.selection.bg_fill = c.linear_multiply(0.3);
        v.selection.stroke.color = c;
        v.hyperlink_color = c;

        v.widgets.hovered.bg_stroke.color = c;
        v.widgets.active.bg_stroke.color = c;
        v.widgets.active.bg_fill = c.linear_multiply(0.4);
    }

    if let Some(c) = to_col(&t.border) {
        v.window_stroke.color = c;
        v.widgets.noninteractive.bg_stroke.color = c;
        v.widgets.inactive.bg_stroke.color = c;
    }

    if let Some(c) = to_col(&t.warn) {
        v.warn_fg_color = c;
    }
    if let Some(c) = to_col(&t.error) {
        v.error_fg_color = c;
    }

    if let Some(r) = t.corner_radius {
        let cr = CornerRadius::same(r);
        v.window_corner_radius = cr;
        v.menu_corner_radius = cr;

        v.widgets.noninteractive.corner_radius = cr;
        v.widgets.inactive.corner_radius = cr;
        v.widgets.hovered.corner_radius = cr;
        v.widgets.active.corner_radius = cr;
        v.widgets.open.corner_radius = cr;
    }

    ctx.set_visuals(v);
}

fn hex_to_color(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        Color32::from_rgb(r, g, b)
    } else {
        eprintln!("Color inválido: {}", hex);
        Color32::MAGENTA
    }
}

fn load_system_font(ctx: &Context, font_family_name: &str) {
    let property = FontPropertyBuilder::new().family(font_family_name).build();
    if let Some((data, _)) = system_fonts::get(&property) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            font_family_name.to_owned(),
            Arc::new(FontData::from_owned(data)),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_family_name.to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_family_name.to_owned());
        ctx.set_fonts(fonts);
        println!("Fuente cargada: {}", font_family_name);
    } else {
        eprintln!("Fuente NO encontrada: {}", font_family_name);
    }
}

fn setup_font_sizes(ctx: &Context, base_size: f32) {
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(base_size * 1.8, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(base_size, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(base_size, FontFamily::Monospace),
        ),
        (
            TextStyle::Button,
            FontId::new(base_size, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(base_size * 0.8, FontFamily::Proportional),
        ),
    ]
    .into();
    style.spacing.item_spacing = egui::vec2(8.0, 5.0);
    ctx.set_style(style);
}

use directories::BaseDirs;
use egui::{
    style::{Selection, WidgetVisuals, Widgets},
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke,
    TextStyle, Visuals,
};
use font_loader::system_fonts::{self, FontPropertyBuilder};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
struct ThemeConfig {
    bg_window: String,
    bg_panel: String,
    bg_input: String,
    bg_faint: String,

    text_primary: String,
    text_secondary: String,

    accent: String,
    border: String,

    success: String,
    error: String,
    warn: String,

    font_family: Option<String>,
    base_font_size: f32,
    corner_radius: u8,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            bg_window: "#1b1b1b".to_string(),
            bg_panel: "#252525".to_string(),
            bg_input: "#151515".to_string(),
            bg_faint: "#202020".to_string(),
            text_primary: "#e0e0e0".to_string(),
            text_secondary: "#909090".to_string(),
            accent: "#7daea3".to_string(),
            border: "#404040".to_string(),
            success: "#a3be8c".to_string(),
            error: "#bf616a".to_string(),
            warn: "#ebcb8b".to_string(),
            font_family: None,
            base_font_size: 14.0,
            corner_radius: 4,
        }
    }
}

pub fn load_and_apply_theme(ctx: &Context) {
    let binding = BaseDirs::new().unwrap();
    let config_dir = binding.config_dir();
    let theme_path = config_dir.join("marmol").join("theme.json");

    println!("Intentando cargar tema desde: {:?}", theme_path);

    let theme_config: ThemeConfig = if theme_path.exists() {
        match fs::read_to_string(&theme_path) {
            Ok(data) => {
                println!("Archivo encontrado. Parseando...");
                match serde_json::from_str(&data) {
                    Ok(cfg) => {
                        println!("Tema cargado exitosamente.");
                        cfg
                    }
                    Err(e) => {
                        eprintln!("Error de sintaxis en theme.json: {}. Usando default.", e);
                        ThemeConfig::default()
                    }
                }
            }
            Err(_) => ThemeConfig::default(),
        }
    } else {
        println!("No existe theme.json. Creando uno nuevo.");
        let def = ThemeConfig::default();
        // Crear carpeta si no existe
        if let Some(parent) = theme_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&def) {
            let _ = fs::write(&theme_path, json);
        }
        def
    };

    let visuals = create_complete_visuals(&theme_config);
    ctx.set_visuals(visuals);

    if let Some(font_name) = &theme_config.font_family {
        load_system_font(ctx, font_name);
    }

    setup_font_sizes(ctx, theme_config.base_font_size);
}

fn create_complete_visuals(t: &ThemeConfig) -> Visuals {
    let bg_win = hex_to_color(&t.bg_window);
    let bg_panel = hex_to_color(&t.bg_panel);
    let bg_input = hex_to_color(&t.bg_input);
    let bg_faint = hex_to_color(&t.bg_faint);

    let txt_pri = hex_to_color(&t.text_primary);

    let accent = hex_to_color(&t.accent);
    let border = hex_to_color(&t.border);

    let err = hex_to_color(&t.error);
    let warn = hex_to_color(&t.warn);

    let radius = CornerRadius::same(t.corner_radius);

    let widgets = Widgets {
        noninteractive: WidgetVisuals {
            weak_bg_fill: bg_win,
            bg_fill: bg_win,
            bg_stroke: Stroke::new(1.0, border),
            fg_stroke: Stroke::new(1.0, txt_pri),
            corner_radius: radius,
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            weak_bg_fill: bg_panel,
            bg_fill: bg_panel,
            bg_stroke: Stroke::new(1.0, border),
            fg_stroke: Stroke::new(1.0, txt_pri),
            corner_radius: radius,
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            weak_bg_fill: accent.linear_multiply(0.2),
            bg_fill: accent.linear_multiply(0.2),
            bg_stroke: Stroke::new(1.0, accent),
            fg_stroke: Stroke::new(1.5, txt_pri),
            corner_radius: radius,
            expansion: 1.0,
        },
        active: WidgetVisuals {
            weak_bg_fill: accent.linear_multiply(0.4),
            bg_fill: accent.linear_multiply(0.4),
            bg_stroke: Stroke::new(2.0, accent),
            fg_stroke: Stroke::new(2.0, Color32::WHITE),
            corner_radius: radius,
            expansion: 1.0,
        },
        open: WidgetVisuals {
            weak_bg_fill: bg_panel,
            bg_fill: bg_panel,
            bg_stroke: Stroke::new(1.0, border),
            fg_stroke: Stroke::new(1.0, txt_pri),
            corner_radius: radius,
            expansion: 0.0,
        },
    };

    let selection = Selection {
        bg_fill: accent.linear_multiply(0.3),
        stroke: Stroke::new(1.0, accent),
    };

    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(txt_pri);
    visuals.widgets = widgets;
    visuals.selection = selection;
    visuals.window_fill = bg_win;
    visuals.panel_fill = bg_panel;
    visuals.extreme_bg_color = bg_input;
    visuals.faint_bg_color = bg_faint;
    visuals.code_bg_color = bg_input;
    visuals.warn_fg_color = warn;
    visuals.error_fg_color = err;
    visuals.hyperlink_color = accent;

    visuals.window_corner_radius = radius;
    visuals.menu_corner_radius = radius;
    visuals.window_stroke = Stroke::new(1.0, border);

    visuals.window_shadow = Shadow::default();
    visuals.popup_shadow = Shadow::default();

    visuals.text_edit_bg_color = Some(bg_input);

    visuals
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

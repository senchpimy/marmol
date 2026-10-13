use super::data::{CustomGroup, Graph};
use crate::main_area::content_enum::Content;
use std::collections::HashSet;

use egui::*;
use egui_plot::{Arrows, Line, MarkerShape, Plot, PlotBounds, PlotPoint, Points, Text};
use std::path::Path;

pub fn draw_ui(
    graph: &mut Graph,
    ui: &mut Ui,
    current_file: &mut String,
    content: &mut Content,
    vault: &str,
    seed_id: Id,
) -> Response {
    if !graph.loaded {
        graph.update_vault(Path::new(vault));
    }
    graph.simulate_physics();
    ui.ctx().request_repaint();

    let animation_duration = 0.5;

    // Igual que Obsidian: sólo se hace pan cuando el puntero no está sobre un
    // nodo (el estado del hover se conserva del frame anterior).
    let allow_pan = graph.dragged_node_index.is_none() && graph.hovered_node_index.is_none();

    let markers_plot = Plot::new(seed_id.with("markers_plot"))
        .data_aspect(1.0)
        .allow_drag(allow_pan)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .allow_double_click_reset(false)
        .show_crosshair(false)
        .show_axes([false, false])
        .show_grid([false, false])
        .include_x(100.0)
        .include_x(-100.0)
        .include_y(100.0)
        .include_y(-100.0);

    let response = markers_plot
        .show(ui, |plot_ui| {
            if graph.points.is_empty() {
                return;
            }

            // --- Zoom con la rueda (paridad con Obsidian) ---
            // Sólo cuando el puntero está sobre el propio grafo, para no hacer
            // zoom mientras se desplaza el panel de controles.
            let scroll = if plot_ui.response().contains_pointer() {
                plot_ui.ctx().input(|i| i.smooth_scroll_delta.y)
            } else {
                0.0
            };
            if scroll.abs() > 0.1 {
                let factor = (scroll * 0.0015).exp();
                plot_ui.zoom_bounds_around_hovered(Vec2::splat(factor));
            }

            // --- Fit-to-view ---
            if graph.pending_fit {
                fit_to_view(graph, plot_ui);
                graph.pending_fit = false;
            }

            let transform = plot_ui.transform();

            // --- Hit testing en espacio de pantalla ---
            let pointer_screen = plot_ui.ctx().input(|i| i.pointer.hover_pos());
            let mut hovered: Option<(usize, f32)> = None;
            if let Some(ps) = pointer_screen {
                for index in 0..graph.points.len() {
                    if !graph.is_visible(index) {
                        continue;
                    }
                    let node_screen = transform.position_from_point(&PlotPoint::new(
                        graph.points_coord[index].0 as f64,
                        graph.points_coord[index].1 as f64,
                    ));
                    let dist = ps.distance(node_screen);
                    if dist <= graph.node_radius(index) + 4.0 {
                        if hovered.map_or(true, |(_, d)| dist < d) {
                            hovered = Some((index, dist));
                        }
                    }
                }
            }
            graph.hovered_node_index = hovered.map(|(i, _)| i);

            let dvalue_dy = transform.dvalue_dpos()[1].abs();

            let mut connected_indices = HashSet::new();
            if let Some(hovered_idx) = graph.hovered_node_index {
                connected_indices.insert(hovered_idx);
                for &(a, b) in &graph.edges {
                    if a == hovered_idx {
                        connected_indices.insert(b);
                    }
                    if b == hovered_idx {
                        connected_indices.insert(a);
                    }
                }
            }
            let is_hovering = graph.hovered_node_index.is_some();

            // --- 1. RENDERIZADO DE LÍNEAS / FLECHAS (CAPA INFERIOR) ---
            let theme = plot_ui.ctx().theme();
            let visuals = plot_ui.ctx().style_of(theme).visuals.clone();
            let base_line_color = visuals.window_stroke.color.linear_multiply(100.0 / 255.0);
            let highlight_color = visuals.widgets.hovered.bg_stroke.color.linear_multiply(220.0 / 255.0);

            let edge_is_highlighted = |a: usize, b: usize| -> bool {
                match graph.hovered_node_index {
                    Some(h) => {
                        (a == h && connected_indices.contains(&b))
                            || (b == h && connected_indices.contains(&a))
                    }
                    None => false,
                }
            };

            if graph.show_arrows && !is_hovering {
                let mut origins = vec![];
                let mut tips = vec![];
                for &(idx_a, idx_b) in &graph.edges {
                    if graph.is_visible(idx_a) && graph.is_visible(idx_b) {
                        origins.push([
                            graph.points_coord[idx_a].0 as f64,
                            graph.points_coord[idx_a].1 as f64,
                        ]);
                        tips.push([
                            graph.points_coord[idx_b].0 as f64,
                            graph.points_coord[idx_b].1 as f64,
                        ]);
                    }
                }
                plot_ui.arrows(
                    Arrows::new("".to_string(), origins, tips)
                        .tip_length(12.0)
                        .color(base_line_color),
                );
            } else {
                for (edge_idx, &(idx_a, idx_b)) in graph.edges.iter().enumerate() {
                    if !(graph.is_visible(idx_a) && graph.is_visible(idx_b)) {
                        continue;
                    }
                    let p1 = graph.points_coord[idx_a];
                    let p2 = graph.points_coord[idx_b];

                    let highlighted = edge_is_highlighted(idx_a, idx_b);
                    let edge_anim_id = seed_id.with("edge_h").with(edge_idx);
                    let edge_t =
                        plot_ui.ctx().animate_bool_with_time(edge_anim_id, highlighted, animation_duration);

                    let mut line_color = base_line_color;
                    let mut width = graph.line_thickness;
                    if is_hovering {
                        if highlighted || edge_t > 0.0 {
                            line_color = lerp_color(base_line_color, highlight_color, edge_t);
                            width = graph.line_thickness * (1.0 + 0.8 * edge_t);
                        } else {
                            line_color = line_color.linear_multiply(0.1);
                        }
                    }

                    plot_ui.line(
                        Line::new(
                            "".to_string(),
                            vec![[p1.0 as f64, p1.1 as f64], [p2.0 as f64, p2.1 as f64]],
                        )
                        .color(line_color)
                        .width(width),
                    );
                }
            }

            // --- 2. RENDERIZADO DE PUNTOS Y TEXTO (CAPA SUPERIOR) ---
            let is_double_click = plot_ui.response().double_clicked();
            let is_drag_started = plot_ui.response().drag_started();
            let is_drag_released = plot_ui.ctx().input(|i| i.pointer.any_released());

            if is_drag_released {
                graph.dragged_node_index = None;
            }

            let span = {
                let bounds = plot_ui.plot_bounds();
                (bounds.max()[1] - bounds.min()[1]).max(0.001)
            };

            let text_visible_alpha = if graph.text_zoom_threshold <= 0.0 {
                1.0
            } else {
                (1.0 - span / graph.text_zoom_threshold).clamp(0.0, 1.0) as f32
            };

            for (index, point) in graph.points.iter().enumerate() {
                if !graph.is_visible(index) {
                    continue;
                }

                let is_node_highlighted = if is_hovering {
                    connected_indices.contains(&index)
                } else {
                    true
                };

                let node_anim_id = seed_id.with("node_h").with(index);
                let node_t = plot_ui
                    .ctx()
                    .animate_bool_with_time(node_anim_id, is_node_highlighted, animation_duration);

                let mut point_color = graph.get_color_for_node(point);
                let point_multiplier = 0.25 + 0.75 * node_t;
                point_color = point_color.linear_multiply(point_multiplier);

                let coords = [
                    graph.points_coord[index].0 as f64,
                    graph.points_coord[index].1 as f64,
                ];

                let radius = graph.node_radius(index);

                let shape = MarkerShape::Circle;

                // Borde para nodos fantasma (archivos que no existen).
                if !point.exists {
                    plot_ui.points(
                        Points::new("".to_string(), coords)
                            .radius(radius + 1.0)
                            .color(visuals.error_fg_color)
                            .shape(shape),
                    );
                }

                plot_ui.points(
                    Points::new("".to_string(), coords)
                        .radius(radius)
                        .color(Color32::from_rgba_unmultiplied(
                            point_color.r(),
                            point_color.g(),
                            point_color.b(),
                            255,
                        ))
                        .shape(shape),
                );

                // Etiqueta: se desvanece al alejar el zoom, pero siempre visible
                // para el nodo bajo el cursor y sus vecinos.
                let mut text_alpha = text_visible_alpha;
                if is_hovering && connected_indices.contains(&index) {
                    text_alpha = text_alpha.max(node_t);
                }
                if text_alpha > 0.01 {
                    let text_color = visuals
                        .override_text_color
                        .unwrap_or(visuals.widgets.inactive.fg_stroke.color)
                        .linear_multiply(text_alpha.max(0.15));

                    let dy = (radius + 4.0) as f64 * dvalue_dy;
                    let texto = Text::new(
                        "".to_string(),
                        PlotPoint::new(
                            graph.points_coord[index].0 as f64,
                            (graph.points_coord[index].1 as f64) - dy,
                        ),
                        RichText::new(&point.text)
                            .size(12.0)
                            .color(text_color),
                    )
                    .anchor(Align2::CENTER_TOP);
                    plot_ui.text(texto);
                }

                if graph.hovered_node_index == Some(index) {
                    if is_double_click && graph.dragged_node_index.is_none() {
                        if point.exists && !point.is_attachment && !point.is_tag {
                            *current_file = point.abs_path.clone();
                            *content = Content::View;
                        }
                    }
                    if is_drag_started {
                        graph.dragged_node_index = Some(index);
                    }
                }
            }

            if let Some(idx) = graph.dragged_node_index {
                if let Some(ptr) = pointer_screen.map(|p| plot_ui.plot_from_screen(p)) {
                    graph.points_coord[idx].0 = ptr.x as f32;
                    graph.points_coord[idx].1 = ptr.y as f32;
                    graph.velocities[idx] = Vec2::ZERO;
                }
            }
        })
        .response;

    // --- Menú contextual sobre un nodo (paridad con Obsidian) ---
    // Al abrir con clic derecho fijamos el nodo objetivo para que el menú siga
    // mostrándose aunque el puntero se mueva sobre el propio menú.
    if response.secondary_clicked() {
        graph.selected_node_index = graph.hovered_node_index;
    }
    response.context_menu(|ui| {
        if let Some(idx) = graph.selected_node_index.or(graph.hovered_node_index) {
            if idx < graph.points.len() {
                let point = graph.points[idx].clone();
                ui.label(RichText::new(&point.text).strong());
                if !point.exists {
                    ui.label(RichText::new("(no existe)").italics().weak());
                }
                ui.separator();

                let openable = point.exists && !point.is_attachment && !point.is_tag;
                if ui
                    .add_enabled(openable, egui::Button::new("Abrir"))
                    .clicked()
                {
                    *current_file = point.abs_path.clone();
                    *content = Content::View;
                    ui.close();
                }
                if ui.button("Copiar ruta").clicked() {
                    ui.ctx().copy_text(point.abs_path.clone());
                    ui.close();
                }
                if !point.links.is_empty() {
                    ui.separator();
                    ui.label(format!("{} enlaces salientes", point.links.len()));
                }
            }
        } else {
            ui.close();
        }
    });

    let plot_rect = response.rect;
    let mut controls_changed = false;

    egui::Area::new(seed_id.with("graph_controls_overlay"))
        .fixed_pos(plot_rect.min + egui::vec2(10.0, 10.0))
        .order(Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .shadow(Shadow::default())
                .fill(ui.style().visuals.window_fill().linear_multiply(0.95))
                .stroke(ui.style().visuals.window_stroke())
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_max_width(240.0);
                    ui.set_max_height(plot_rect.height() - 40.0);

                    // Barra superior con acciones rápidas.
                    ui.horizontal(|ui| {
                        ui.toggle_value(&mut graph.show_controls, "⚙");
                        if ui.button("Ajustar").clicked() {
                            graph.pending_fit = true;
                        }
                        if ui
                            .button("⟳")
                            .on_hover_text("Recargar el grafo desde el vault")
                            .clicked()
                        {
                            graph.loaded = false;
                        }
                        if ui.button("Restablecer").on_hover_text("Ajustes por defecto").clicked() {
                            reset_settings(graph);
                            controls_changed = true;
                        }
                    });

                    if graph.show_controls {
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_salt(seed_id.with("graph_ctrl_scroll"))
                            .show(ui, |ui| {
                                if draw_controls(graph, ui, seed_id) {
                                    controls_changed = true;
                                }
                            });
                    }
                });
        });

    if controls_changed {
        graph.settings_dirty = true;
    }
    // Guardar al soltar el puntero, para no escribir en disco en cada frame
    // mientras se arrastra un slider.
    if graph.settings_dirty && !ui.ctx().input(|i| i.pointer.primary_down()) {
        graph.save_settings();
        graph.settings_dirty = false;
    }

    response
}

fn fit_to_view(graph: &Graph, plot_ui: &mut egui_plot::PlotUi<'_>) {
    let mut min = (f32::INFINITY, f32::INFINITY);
    let mut max = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    let mut any = false;
    for index in 0..graph.points.len() {
        if !graph.is_visible(index) {
            continue;
        }
        let (x, y) = graph.points_coord[index];
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        min.0 = min.0.min(x);
        min.1 = min.1.min(y);
        max.0 = max.0.max(x);
        max.1 = max.1.max(y);
        any = true;
    }
    if !any {
        return;
    }
    let pad_x = ((max.0 - min.0) * 0.15).max(20.0);
    let pad_y = ((max.1 - min.1) * 0.15).max(20.0);
    plot_ui.set_plot_bounds(PlotBounds::from_min_max(
        [(min.0 - pad_x) as f64, (min.1 - pad_y) as f64],
        [(max.0 + pad_x) as f64, (max.1 + pad_y) as f64],
    ));
}

fn reset_settings(graph: &mut Graph) {
    graph.center_force = 0.15;
    graph.repel_force = 30.0;
    graph.link_force = 0.8;
    graph.link_distance = 30.0;
    graph.filter_query.clear();
    graph.show_attachments = true;
    graph.show_existing_only = false;
    graph.show_orphans = true;
    graph.show_tags = false;
    graph.show_arrows = false;
    graph.text_zoom_threshold = 500.0;
    graph.node_size = 7.0;
    graph.line_thickness = 2.0;
    graph.custom_groups.clear();
}

fn draw_controls(graph: &mut Graph, ui: &mut Ui, seed_id: Id) -> bool {
    let mut changed = false;

    ui.push_id(seed_id, |ui| {
        ui.collapsing("Filtros", |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut graph.filter_query)
                            .hint_text("file:, path:, tag:, line:, section:")
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            if ui
                .add_enabled(!graph.filter_query.is_empty(), egui::Button::new("Limpiar"))
                .clicked()
            {
                graph.filter_query.clear();
                changed = true;
            }

            ui.separator();
            if ui.checkbox(&mut graph.show_tags, "Tags").changed() {
                changed = true;
            }
            if ui.checkbox(&mut graph.show_attachments, "Adjuntos").changed() {
                changed = true;
            }
            if ui
                .checkbox(&mut graph.show_existing_only, "Sólo archivos existentes")
                .changed()
            {
                changed = true;
            }
            if ui.checkbox(&mut graph.show_orphans, "Huérfanos").changed() {
                changed = true;
            }
        });

        ui.collapsing("Grupos", |ui| {
            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut graph.new_group_col,
                    egui::widgets::color_picker::Alpha::Opaque,
                );
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut graph.new_group_query)
                            .hint_text("file:, path:, tag:, line:, section:")
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            if ui.button("Agregar grupo").clicked() && !graph.new_group_query.is_empty() {
                graph.custom_groups.push(CustomGroup::new(
                    std::mem::take(&mut graph.new_group_query),
                    graph.new_group_col,
                ));
                changed = true;
            }

            ui.separator();
            let mut index_to_remove = None;
            for (i, group) in graph.custom_groups.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut group.enabled, "").changed() {
                        changed = true;
                    }
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, group.color);
                    ui.label(group.query.clone());
                    if ui.button("x").clicked() {
                        index_to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = index_to_remove {
                graph.custom_groups.remove(i);
                changed = true;
            }
            if graph.custom_groups.is_empty() {
                ui.weak("Sin grupos");
            }
        });

        ui.collapsing("Visualización", |ui| {
            if ui.checkbox(&mut graph.show_arrows, "Flechas").changed() {
                changed = true;
            }
            if ui
                .add(
                    egui::Slider::new(&mut graph.text_zoom_threshold, 0.0..=2000.0)
                        .text("Umbral de texto")
                        .logarithmic(true),
                )
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut graph.node_size, 2.0..=20.0).text("Tamaño de nodo"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(
                    egui::Slider::new(&mut graph.line_thickness, 0.5..=10.0)
                        .text("Grosor de enlace"),
                )
                .changed()
            {
                changed = true;
            }
        });

        ui.collapsing("Fuerzas", |ui| {
            if ui
                .add(egui::Slider::new(&mut graph.center_force, 0.0..=1.0).text("Fuerza central"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut graph.repel_force, 0.0..=100.0).text("Fuerza de repulsión"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut graph.link_force, 0.0..=3.0).text("Fuerza de enlace"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut graph.link_distance, 0.0..=300.0).text("Distancia de enlace"))
                .changed()
            {
                changed = true;
            }

            ui.separator();
            ui.weak("Avanzado");
            if ui
                .add(egui::Slider::new(&mut graph.group_force, 0.0..=5.0).text("Agrupación"))
                .changed()
            {
                changed = true;
            }
            if ui
                .add(egui::Slider::new(&mut graph.tag_force, 0.1..=10.0).text("Atracción de tags"))
                .changed()
            {
                changed = true;
            }
            ui.horizontal(|ui| {
                if color_picker::color_edit_button_srgba(
                    ui,
                    &mut graph.orphan_color,
                    egui::widgets::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    changed = true;
                }
                ui.label("Color de huérfanos");
            });
        });
    });

    changed
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_premultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

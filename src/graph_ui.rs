use crate::graph_state::{CustomGroup, Graph, MatchType};
use crate::main_area::content_enum::Content;
use std::collections::HashSet;

use egui::*;
use egui_plot::{Arrows, Line, MarkerShape, Plot, PlotPoint, Points, Text};
use std::path::Path;

pub fn draw_ui(
    graph: &mut Graph,
    ui: &mut Ui,
    current_file: &mut String,
    content: &mut Content,
    vault: &str,
) -> Response {
    graph.simulate_physics();
    ui.ctx().request_repaint();

    let markers_plot = Plot::new("Graph")
        .data_aspect(1.0)
        .allow_drag(graph.dragged_node_index.is_none())
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

            let pointer_pos = plot_ui.pointer_coordinate();

            graph.hovered_node_index = None;
            if let Some(ptr) = pointer_pos {
                for (index, _) in graph.points.iter().enumerate() {
                    if !graph.is_visible(index) {
                        continue;
                    }
                    if is_close(ptr, graph.points_coord[index], 1.5) {
                        graph.hovered_node_index = Some(index);
                        break;
                    }
                }
            }

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

            let base_line_color = plot_ui.ctx().style().visuals.window_stroke.color.linear_multiply(100.0 / 255.0);

            if graph.show_arrows {
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

                let mut arrows = Arrows::new("".to_string(), origins, tips).tip_length(25.0);

                if is_hovering {
                    arrows = arrows.color(base_line_color.linear_multiply(0.2));
                } else {
                    arrows = arrows.color(base_line_color);
                }
                plot_ui.arrows(arrows);
            } else {
                for &(idx_a, idx_b) in &graph.edges {
                    if graph.is_visible(idx_a) && graph.is_visible(idx_b) {
                        let p1 = graph.points_coord[idx_a];
                        let p2 = graph.points_coord[idx_b];

                        let mut line_color = base_line_color;

                        if is_hovering {
                            let a_connected = connected_indices.contains(&idx_a);
                            let b_connected = connected_indices.contains(&idx_b);

                            if a_connected && b_connected {
                                if idx_a == graph.hovered_node_index.unwrap()
                                    || idx_b == graph.hovered_node_index.unwrap()
                                {
                                                                        line_color = plot_ui.ctx().style().visuals.widgets.hovered.bg_stroke.color.linear_multiply(180.0 / 255.0);
                                }
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
                            .width(graph.line_thickness),
                        );
                    }
                }
            }

            let is_double_click = plot_ui.response().double_clicked();
            let is_drag_started = plot_ui.response().drag_started();
            let is_drag_released = plot_ui.ctx().input(|i| i.pointer.any_released());

            if is_drag_released {
                graph.dragged_node_index = None;
            }

            for (index, point) in graph.points.iter().enumerate() {
                if !graph.is_visible(index) {
                    continue;
                }

                let mut point_color = graph.get_color_for_node(point);

                if is_hovering {
                    if connected_indices.contains(&index) {
                        point_color = point_color.linear_multiply(1.0);
                    } else {
                        point_color = point_color.linear_multiply(0.3);
                    }
                }

                let coords = [
                    graph.points_coord[index].0 as f64,
                    graph.points_coord[index].1 as f64,
                ];

                let mut radius = graph.node_size;
                if point.is_tag {
                    let degree = graph.node_degrees[index] as f32;
                    radius = (graph.node_size * 1.5 + degree * 0.5).min(50.0);
                } else if point.is_attachment {
                    radius = graph.node_size * 0.7;
                } else if !point.exists {
                    radius = graph.node_size * 0.85;
                }

                let mut shape = MarkerShape::Circle;
                if point.is_attachment {
                    shape = MarkerShape::Square;
                }
                if point.is_tag {
                    shape = MarkerShape::Diamond;
                }

                plot_ui.points(
                    Points::new("".to_string(), coords)
                        .radius(radius)
                        .color(point_color)
                        .shape(shape),
                );

                let bounds = plot_ui.plot_bounds();
                let diff = bounds.max()[1] - bounds.min()[1];

                if diff < graph.text_zoom_threshold {
                    let mut text_color = plot_ui.ctx().style().visuals.widgets.inactive.fg_stroke.color;

                    if is_hovering {
                        if connected_indices.contains(&index) {
                            text_color = plot_ui.ctx().style().visuals.override_text_color.unwrap_or(Color32::WHITE);
                        } else {
                            text_color = text_color.linear_multiply(0.2); // Ocultar casi todo el resto
                        }
                    }

                    let texto = Text::new(
                        "".to_string(),
                        PlotPoint::new(
                            graph.points_coord[index].0 as f64,
                            (graph.points_coord[index].1 as f64) - (diff * 0.02),
                        ),
                        RichText::new(&point.text).size(12.0).color(text_color),
                    );
                    plot_ui.text(texto);
                }

                if let Some(hovered) = graph.hovered_node_index {
                    if hovered == index {
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
            }

            if let Some(idx) = graph.dragged_node_index {
                if let Some(ptr) = pointer_pos {
                    graph.points_coord[idx].0 = ptr.x as f32;
                    graph.points_coord[idx].1 = ptr.y as f32;
                    graph.velocities[idx] = Vec2::ZERO;
                }
            }
        })
        .response;

    let plot_rect = response.rect;
    let mut controls_changed = false;

    egui::Area::new("graph_controls_overlay".into())
        .fixed_pos(plot_rect.min + egui::vec2(10.0, 10.0))
        .order(Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .shadow(Shadow::default())
                .fill(ui.style().visuals.window_fill().linear_multiply(0.95))
                .stroke(ui.style().visuals.window_stroke())
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_max_width(220.0);
                    ui.set_max_height(plot_rect.height() - 40.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if draw_controls(graph, ui) {
                            controls_changed = true;
                        }
                    });
                });
        });

    if controls_changed {
        graph.update_vault(Path::new(vault));
    }

    response
}

fn draw_controls(graph: &mut Graph, ui: &mut Ui) -> bool {
    let mut changed = false;

    ui.collapsing("Configuración Física", |ui| {
        ui.add(egui::Slider::new(&mut graph.repel_force, 1.0..=100.0).text("Repulsión"));
        ui.add(egui::Slider::new(&mut graph.link_force, 0.1..=3.0).text("Links"));
        ui.add(egui::Slider::new(&mut graph.group_force, 0.0..=5.0).text("Agrupación"));
        ui.add(egui::Slider::new(&mut graph.center_force, 0.01..=1.0).text("Gravedad"));
        ui.add(egui::Slider::new(&mut graph.tag_force, 0.1..=10.0).text("Atracción Tags"));

        ui.horizontal(|ui| {
            color_picker::color_edit_button_srgba(
                ui,
                &mut graph.orphan_color,
                egui::widgets::color_picker::Alpha::Opaque,
            );
            ui.label("Color Huérfanos");
        });
    });

    ui.collapsing("Visualización", |ui| {
        ui.checkbox(&mut graph.show_arrows, "Mostrar Flechas (Dirección)");

        ui.add(egui::Slider::new(&mut graph.node_size, 2.0..=20.0).text("Tamaño Nodo"));
        ui.add(egui::Slider::new(&mut graph.line_thickness, 0.5..=10.0).text("Grosor Línea"));

        ui.label("Visibilidad de Texto (Zoom):");
        ui.add(
            egui::Slider::new(&mut graph.text_zoom_threshold, 10.0..=2000.0)
                .text("Umbral")
                .logarithmic(true),
        );
    });

    ui.collapsing("Crear Grupos", |ui| {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("group_type_combo")
                .selected_text(match graph.new_group_type {
                    MatchType::Tag => "Tag",
                    MatchType::Filename => "Filename",
                    MatchType::Path => "Path",
                    MatchType::Content => "Content",
                    MatchType::Section => "Section",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut graph.new_group_type, MatchType::Tag, "Tag");
                    ui.selectable_value(&mut graph.new_group_type, MatchType::Filename, "Filename");
                    ui.selectable_value(&mut graph.new_group_type, MatchType::Path, "Path");
                    ui.selectable_value(&mut graph.new_group_type, MatchType::Content, "Content");
                    ui.selectable_value(&mut graph.new_group_type, MatchType::Section, "Section");
                });

            color_picker::color_edit_button_srgba(
                ui,
                &mut graph.new_group_col,
                egui::widgets::color_picker::Alpha::Opaque,
            );
        });

        ui.text_edit_singleline(&mut graph.new_group_val)
            .on_hover_text("Valor a buscar");

        if ui.button("Agregar Grupo").clicked() {
            if !graph.new_group_val.is_empty() {
                graph.custom_groups.push(CustomGroup {
                    match_type: graph.new_group_type.clone(),
                    value: graph.new_group_val.clone(),
                    color: graph.new_group_col,
                });
                graph.new_group_val.clear();
            }
        }

        ui.separator();
        ui.label("Grupos Activos:");
        let mut index_to_remove = None;
        for (i, group) in graph.custom_groups.iter().enumerate() {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, group.color);
                let type_str = match group.match_type {
                    MatchType::Tag => "Tag",
                    MatchType::Filename => "File",
                    MatchType::Path => "Path",
                    MatchType::Content => "Cont",
                    MatchType::Section => "Sect",
                };
                ui.label(format!("{}: '{}'", type_str, group.value));
                if ui.button("x").clicked() {
                    index_to_remove = Some(i);
                }
            });
        }
        if let Some(i) = index_to_remove {
            graph.custom_groups.remove(i);
        }
    });

    ui.collapsing("Filtros", |ui| {
        ui.label("Filename:");
        ui.text_edit_singleline(&mut graph.filter_filename);
        ui.label("Tag:");
        ui.text_edit_singleline(&mut graph.filter_tag);
        ui.label("Path:");
        ui.text_edit_singleline(&mut graph.filter_path);
        ui.label("Line:");
        ui.text_edit_singleline(&mut graph.filter_line);
        ui.label("Section:");
        ui.text_edit_singleline(&mut graph.filter_section);

        ui.separator();
        ui.checkbox(&mut graph.show_attachments, "Mostrar Adjuntos");
        ui.checkbox(&mut graph.show_existing_only, "Ocultar Nodos Fantasma");
        ui.checkbox(&mut graph.show_orphans, "Mostrar Huérfanos");

        if ui
            .checkbox(&mut graph.show_tags, "Mostrar Nodos de Tags")
            .changed()
        {
            changed = true;
        }

        if ui.button("Limpiar Todo").clicked() {
            graph.filter_filename.clear();
            graph.filter_tag.clear();
            graph.filter_path.clear();
            graph.filter_line.clear();
            graph.filter_section.clear();
        }
    });

    changed
}

fn is_close(delta: PlotPoint, point_pos: (f32, f32), tol: f32) -> bool {
    let dx = (delta.x as f32 - point_pos.0).abs();
    let dy = (delta.y as f32 - point_pos.1).abs();
    dx < tol && dy < tol
}

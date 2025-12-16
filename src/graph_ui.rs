// graph_ui.rs
use crate::graph_state::{CustomGroup, Graph, MatchType}; // Importar desde donde guardaste el archivo anterior
use crate::main_area::content_enum::Content;

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
    // Calcular física
    graph.simulate_physics();
    ui.ctx().request_repaint();

    // Configurar Plot
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

            // Dibujar Conexiones
            let line_color = Color32::from_rgba_unmultiplied(100, 100, 100, 100);
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
                plot_ui.arrows(
                    Arrows::new("".to_string(), origins, tips)
                        .color(line_color)
                        .tip_length(25.0),
                );
            } else {
                for &(idx_a, idx_b) in &graph.edges {
                    if graph.is_visible(idx_a) && graph.is_visible(idx_b) {
                        let p1 = graph.points_coord[idx_a];
                        let p2 = graph.points_coord[idx_b];
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

            // Manejo de Input en el Plot
            let pointer_pos = plot_ui.pointer_coordinate();
            let is_double_click = plot_ui.response().double_clicked();
            let is_drag_started = plot_ui.response().drag_started();
            let is_drag_released = plot_ui.ctx().input(|i| i.pointer.any_released());

            if is_drag_released {
                graph.dragged_node_index = None;
            }

            // Dibujar Nodos
            for (index, point) in graph.points.iter().enumerate() {
                if !graph.is_visible(index) {
                    continue;
                }

                let point_color = graph.get_color_for_node(point);
                let coords = [
                    graph.points_coord[index].0 as f64,
                    graph.points_coord[index].1 as f64,
                ];

                let mut radius = graph.node_size;
                if point.is_tag {
                    let degree = graph.node_degrees[index] as f32;
                    radius = (graph.node_size * 1.5 + degree * 0.5).min(50.0);
                } else if point.is_attachment {
                    radius *= 0.7;
                } else if !point.exists {
                    radius *= 0.85;
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

                // Dibujar Texto (Zoom dependiente)
                let diff = plot_ui.plot_bounds().max()[1] - plot_ui.plot_bounds().min()[1];
                if diff < graph.text_zoom_threshold {
                    plot_ui.text(Text::new(
                        "".to_string(),
                        PlotPoint::new(coords[0], coords[1] - diff * 0.02),
                        RichText::new(&point.text).size(12.0),
                    ));
                }

                // Interacción con Nodo
                if let Some(ptr) = pointer_pos {
                    if is_close(ptr, graph.points_coord[index], 1.2) {
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

            // Actualizar posición nodo arrastrado
            if let Some(idx) = graph.dragged_node_index {
                if let Some(ptr) = pointer_pos {
                    graph.points_coord[idx].0 = ptr.x as f32;
                    graph.points_coord[idx].1 = ptr.y as f32;
                    graph.velocities[idx] = Vec2::ZERO;
                }
            }
        })
        .response;

    // Dibujar Panel de Controles Flotante
    let plot_rect = response.rect;
    let mut controls_changed = false;

    egui::Area::new("graph_controls_overlay".into())
        .fixed_pos(plot_rect.min + egui::vec2(10.0, 10.0))
        .order(Order::Foreground)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .shadow(Shadow::default())
                .fill(ui.style().visuals.window_fill().linear_multiply(0.95))
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
        ui.add(Slider::new(&mut graph.repel_force, 1.0..=100.0).text("Repulsión"));
        ui.add(Slider::new(&mut graph.link_force, 0.1..=3.0).text("Links"));
        ui.add(Slider::new(&mut graph.group_force, 0.0..=5.0).text("Agrupación"));
        ui.add(Slider::new(&mut graph.center_force, 0.01..=1.0).text("Gravedad"));
        ui.add(Slider::new(&mut graph.tag_force, 0.1..=10.0).text("Atracción Tags"));
        ui.horizontal(|ui| {
            color_picker::color_edit_button_srgba(
                ui,
                &mut graph.orphan_color,
                widgets::color_picker::Alpha::Opaque,
            );
            ui.label("Color Huérfanos");
        });
    });

    ui.collapsing("Visualización", |ui| {
        ui.checkbox(&mut graph.show_arrows, "Mostrar Flechas");
        ui.add(Slider::new(&mut graph.node_size, 2.0..=20.0).text("Tamaño Nodo"));
        ui.add(Slider::new(&mut graph.line_thickness, 0.5..=10.0).text("Grosor Línea"));
        ui.label("Zoom Texto:");
        ui.add(
            Slider::new(&mut graph.text_zoom_threshold, 10.0..=2000.0)
                .text("Umbral")
                .logarithmic(true),
        );
    });

    ui.collapsing("Grupos y Colores", |ui| {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("group_type")
                .selected_text(match graph.new_group_type {
                    MatchType::Tag => "Tag",
                    MatchType::Filename => "File",
                    MatchType::Path => "Path",
                    MatchType::Content => "Cont",
                    MatchType::Section => "Sect",
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
                widgets::color_picker::Alpha::Opaque,
            );
        });
        ui.text_edit_singleline(&mut graph.new_group_val);
        if ui.button("Agregar Grupo").clicked() && !graph.new_group_val.is_empty() {
            graph.custom_groups.push(CustomGroup {
                match_type: graph.new_group_type.clone(),
                value: graph.new_group_val.clone(),
                color: graph.new_group_col,
            });
            graph.new_group_val.clear();
        }

        let mut remove_idx = None;
        for (i, g) in graph.custom_groups.iter().enumerate() {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(r, 2.0, g.color);
                ui.label(&g.value);
                if ui.button("x").clicked() {
                    remove_idx = Some(i);
                }
            });
        }
        if let Some(i) = remove_idx {
            graph.custom_groups.remove(i);
        }
    });

    ui.collapsing("Filtros", |ui| {
        ui.label("Nombre:");
        ui.text_edit_singleline(&mut graph.filter_filename);
        ui.label("Tag:");
        ui.text_edit_singleline(&mut graph.filter_tag);
        ui.label("Path:");
        ui.text_edit_singleline(&mut graph.filter_path);
        ui.label("Contenido:");
        ui.text_edit_singleline(&mut graph.filter_line);
        ui.label("Sección (#):");
        ui.text_edit_singleline(&mut graph.filter_section);

        ui.separator();
        ui.checkbox(&mut graph.show_attachments, "Mostrar Adjuntos");
        ui.checkbox(&mut graph.show_existing_only, "Ocultar Fantasmas");
        ui.checkbox(&mut graph.show_orphans, "Mostrar Huérfanos");
        if ui
            .checkbox(&mut graph.show_tags, "Mostrar Nodos Tags")
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
    (delta.x as f32 - point_pos.0).abs() < tol && (delta.y as f32 - point_pos.1).abs() < tol
}

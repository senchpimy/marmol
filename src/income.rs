use chrono::Local;
use core::ops::RangeInclusive;
use egui::*;
use egui_plot::{GridMark, Line, MarkerShape, PlotPoints, Points};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(PartialEq)]
enum Ventana {
    Normal,
    Graficos,
    Categorias,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
enum TipoMovimiento {
    Ingreso,
    Gasto,
    Null,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Movimiento {
    fecha: String,
    tipo: TipoMovimiento,
    description: String,
    categoria: usize,
    monto: f32,
}

impl Movimiento {
    fn new(
        fecha: String,
        tipo: TipoMovimiento,
        description: String,
        categoria: usize,
        monto: f32,
    ) -> Movimiento {
        Movimiento {
            fecha,
            tipo,
            description,
            categoria,
            monto,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transacciones {
    transacciones: Vec<Movimiento>,
    categorias: Vec<String>,
    colores: Vec<[f32; 3]>,
}

pub fn load_data(path: &str) -> Transacciones {
    if !Path::new(path).exists() {
        return Transacciones {
            transacciones: vec![],
            categorias: vec!["General".to_string()],
            colores: vec![[0.5, 0.5, 0.5]],
        };
    }
    let data = fs::read_to_string(Path::new(path)).expect("Unable to read file");
    let data: Transacciones = serde_json::from_str(&data).unwrap_or(Transacciones {
        transacciones: vec![],
        categorias: vec!["General".to_string()],
        colores: vec![[0.5, 0.5, 0.5]],
    });
    data
}

#[derive(PartialEq)]
enum GraficaVer {
    Grafica,
    Elemento,
}

pub struct IncomeGui {
    json_content: Transacciones,
    path: String,
    categorias: HashMap<usize, i32>,
    valor: usize,
    description: String,
    amount: String,
    fecha: String,
    error: String,
    edit: (i32, TipoMovimiento),
    ventana: Ventana,
    mov_sort: Vec<String>,
    points: Vec<[f64; 2]>,
    lines: Vec<[f64; 2]>,
    ingresos: HashMap<String, f64>,
    cambiar: bool,
    editar_index: i32,
    categorias_string: String,
    max: f64,
    ver_gra: GraficaVer,
    ver_gra_i: usize,
    ingresos_cat: HashMap<usize, f32>,
    gastos_cat: HashMap<usize, f32>,
    ingresos_cat_tot: f32,
    gastos_cat_tot: f32,
}

impl Default for IncomeGui {
    fn default() -> Self {
        Self {
            json_content: Transacciones {
                transacciones: Vec::new(),
                categorias: Vec::new(),
                colores: Vec::new(),
            },
            path: String::new(),
            categorias: HashMap::new(),
            valor: 0,
            description: String::new(),
            amount: String::new(),
            fecha: Local::now().format("%Y-%m-%d").to_string(),
            error: String::new(),
            edit: (-1, TipoMovimiento::Null),
            ventana: Ventana::Normal,
            points: Vec::new(),
            lines: Vec::new(),
            mov_sort: Vec::new(),
            ingresos: HashMap::new(),
            cambiar: false,
            editar_index: -1,
            categorias_string: String::new(),
            max: 0.0,
            ver_gra: GraficaVer::Grafica,
            ver_gra_i: 0,
            ingresos_cat: HashMap::new(),
            gastos_cat: HashMap::new(),
            ingresos_cat_tot: 0.0,
            gastos_cat_tot: 0.0,
        }
    }
}

impl IncomeGui {
    pub fn set_data(&mut self, json_content: Transacciones) {
        self.json_content = json_content;
        self.update_categorias();
        self.get_points();
    }

    fn get_points(&mut self) {
        self.ingresos = HashMap::new();
        self.mov_sort = Vec::new();
        self.points = Vec::new();
        self.lines = Vec::new();
        self.max = 0.0;
        for j in &self.json_content.transacciones {
            self.ingresos
                .entry(j.fecha.clone())
                .and_modify(|x| {
                    if j.tipo == TipoMovimiento::Ingreso {
                        *x += j.monto as f64;
                    } else {
                        *x -= j.monto as f64;
                    }
                })
                .or_insert({
                    if j.tipo == TipoMovimiento::Ingreso {
                        j.monto as f64
                    } else {
                        (j.monto * -1.) as f64
                    }
                });
        }
        for i in self.ingresos.keys() {
            self.mov_sort.push(i.clone());
        }
        self.mov_sort.sort();
        let mut j = 0.0;
        let mut total = 0.0;
        for i in &self.mov_sort {
            if self.ingresos.get(i).unwrap().abs() > self.max {
                self.max = self.ingresos.get(i).unwrap().abs();
            }
            total += *self.ingresos.get(i).unwrap();
            self.points.push([j, total]);
            j += 1.;
        }
        self.lines = self.points.clone();
    }

    fn update_categorias(&mut self) {
        if !self.json_content.transacciones.is_empty() {
            if self.json_content.transacciones[0].categoria < self.json_content.categorias.len() {
                self.valor = self.json_content.transacciones[0].categoria;
            } else {
                self.valor = 0;
            }
        }
        self.categorias = HashMap::new();
        self.ingresos_cat = HashMap::new();
        self.gastos_cat = HashMap::new();
        self.ingresos_cat_tot = 0.0;
        self.gastos_cat_tot = 0.0;

        for elemento in &self.json_content.transacciones {
            self.categorias
                .entry(elemento.categoria)
                .and_modify(|x| *x += 1)
                .or_insert(1);
            if elemento.tipo == TipoMovimiento::Ingreso {
                self.ingresos_cat
                    .entry(elemento.categoria)
                    .and_modify(|x| *x += elemento.monto)
                    .or_insert(elemento.monto);
                self.ingresos_cat_tot += elemento.monto;
            } else {
                self.gastos_cat
                    .entry(elemento.categoria)
                    .and_modify(|x| *x += elemento.monto)
                    .or_insert(elemento.monto);
                self.gastos_cat_tot += elemento.monto;
            }
        }
    }

    pub fn set_path(&mut self, path: &str) {
        if path != self.path {
            self.path = String::from(path);
            self.set_data(load_data(&self.path));
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.header_nav(ui);

        egui::CentralPanel::default().show_inside(ui, |ui| match self.ventana {
            Ventana::Normal => {
                if ui.available_width() > 800.0 {
                    ui.columns(2, |cols| {
                        cols[0].vertical(|ui| self.vista_separada(ui));

                        cols[1].vertical(|ui| {
                            self.add_record(ui);
                            ui.separator();
                            self.categorias(ui);
                        });
                    });
                } else {
                    let available_height = ui.available_height();

                    egui::TopBottomPanel::bottom("controls_bottom")
                        .resizable(true)
                        .default_height(available_height * 0.5)
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.add_record(ui);
                                ui.add_space(20.0);
                                ui.separator();
                                self.categorias(ui);
                            });
                        });

                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        self.vista_separada(ui);
                    });
                }
            }
            Ventana::Graficos => self.grafica(ui),
            Ventana::Categorias => self.canvas(ui),
        });

        self.save();
    }
    fn header_nav(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("nav_panel")
            .frame(
                Frame::default()
                    .fill(ui.visuals().window_fill())
                    .inner_margin(8.0),
            )
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.ventana, Ventana::Normal, "📋 Transacciones");
                    ui.selectable_value(&mut self.ventana, Ventana::Graficos, "📈 Evolución");
                    ui.selectable_value(&mut self.ventana, Ventana::Categorias, "🍩 Distribución");
                });
            });
    }

    fn categorias(&mut self, ui: &mut egui::Ui) {
        ui.heading("Gestión de Categorías");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Editar:");
                egui::ComboBox::from_id_salt("cat_edit_combo")
                    .selected_text(
                        self.json_content
                            .categorias
                            .get(self.valor)
                            .unwrap_or(&"?".to_string()),
                    )
                    .show_ui(ui, |ui| {
                        for (val, key) in self.json_content.categorias.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut self.valor, val, key);
                                if ui.small_button("🗑").on_hover_text("Eliminar").clicked() {
                                    self.editar_index = val as i32;
                                }
                            });
                        }
                    });
            });

            ui.horizontal(|ui| {
                egui::widgets::color_picker::color_edit_button_rgb(
                    ui,
                    &mut self.json_content.colores[self.valor],
                );
                ui.add(egui::TextEdit::singleline(
                    &mut self.json_content.categorias[self.valor],
                ));
            });

            ui.separator();

            ui.add(
                egui::TextEdit::singleline(&mut self.categorias_string)
                    .hint_text("Nombre nueva cat."),
            );

            if ui.button("Crear").clicked() {
                if !self.categorias_string.is_empty() {
                    self.json_content
                        .categorias
                        .push(self.categorias_string.clone());
                    self.json_content.colores.push([0.5, 0.5, 0.5]);

                    self.categorias_string = String::new();

                    self.valor = self.json_content.categorias.len() - 1;
                }
            }

            if self.editar_index != -1 {
                if self.json_content.categorias.len() > 1 {
                    self.json_content
                        .categorias
                        .remove(self.editar_index as usize);
                    self.json_content.colores.remove(self.editar_index as usize);

                    for g in &mut self.json_content.transacciones {
                        if g.categoria == self.editar_index as usize {
                            g.categoria = 0;
                        } else if g.categoria > self.editar_index as usize {
                            g.categoria -= 1;
                        }
                    }
                    self.valor = 0;
                }
                self.editar_index = -1;
            }
        });
    }

    pub fn vista_separada(&mut self, ui: &mut egui::Ui) {
        let mut tot = 0.0;
        let mut remove: i32 = -1;

        let Transacciones {
            transacciones,
            categorias,
            colores,
        } = &mut self.json_content;
        let edit_state = &mut self.edit;
        let cambiar_ref = &mut self.cambiar;

        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.heading(RichText::new("📉 Gastos").color(Color32::from_rgb(230, 80, 80)));
                egui::ScrollArea::vertical()
                    .id_salt("gastos_scroll")
                    .max_height(ui.available_height() - 30.0)
                    .show(ui, |ui| {
                        for (this, elemento) in transacciones.iter_mut().enumerate() {
                            if elemento.tipo == TipoMovimiento::Gasto {
                                tot -= elemento.monto;
                                if *edit_state == (this as i32, TipoMovimiento::Gasto) {
                                    draw_edit_card(
                                        ui,
                                        elemento,
                                        edit_state,
                                        cambiar_ref,
                                        categorias,
                                    );
                                } else {
                                    draw_transaction_card(
                                        ui,
                                        elemento,
                                        &mut remove,
                                        this as i32,
                                        false,
                                        edit_state,
                                        colores,
                                    );
                                }
                            }
                        }
                    });
            });

            // Columna Ingresos
            cols[1].vertical(|ui| {
                ui.heading(RichText::new("📈 Ingresos").color(Color32::from_rgb(80, 200, 80)));
                egui::ScrollArea::vertical()
                    .id_salt("ingresos_scroll")
                    .max_height(ui.available_height() - 30.0)
                    .show(ui, |ui| {
                        for (this, elemento) in transacciones.iter_mut().enumerate() {
                            if elemento.tipo == TipoMovimiento::Ingreso {
                                tot += elemento.monto;
                                if *edit_state == (this as i32, TipoMovimiento::Ingreso) {
                                    draw_edit_card(
                                        ui,
                                        elemento,
                                        edit_state,
                                        cambiar_ref,
                                        categorias,
                                    );
                                } else {
                                    draw_transaction_card(
                                        ui,
                                        elemento,
                                        &mut remove,
                                        this as i32,
                                        true,
                                        edit_state,
                                        colores,
                                    );
                                }
                            }
                        }
                    });
            });
        });

        if remove != -1 {
            self.json_content.transacciones.remove(remove as usize);
            self.update_categorias();
            self.cambiar = true;
        }
        if self.cambiar {
            self.get_points();
            self.update_categorias();
            self.cambiar = false;
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Balance Total:").size(16.0).strong());
            let color = if tot >= 0.0 {
                Color32::from_rgb(100, 220, 100)
            } else {
                Color32::from_rgb(220, 80, 80)
            };
            ui.label(
                RichText::new(format!("{:.2}", tot))
                    .size(16.0)
                    .strong()
                    .color(color),
            );
        });
    }

    pub fn save(&self) {
        if self.path.is_empty() {
            return;
        }
        let file = String::from(&self.path);
        if let Ok(mut file2) = fs::File::create(file) {
            if let Ok(conts) = serde_json::to_string_pretty(&self.json_content) {
                let _ = file2.write_all(conts.as_bytes());
            }
        }
    }

    pub fn add_record(&mut self, ui: &mut egui::Ui) {
        ui.heading("Nuevo Registro");

        Frame::group(ui.style()).inner_margin(8.0).show(ui, |ui| {
            ui.vertical(|ui| {
                if !self.error.is_empty() {
                    ui.label(
                        RichText::new(format!("⚠ {}", self.error))
                            .color(ui.visuals().error_fg_color)
                            .small(),
                    );
                    ui.add_space(4.0);
                }

                ui.horizontal(|ui| {
                    ui.label("📅");
                    ui.add(egui::TextEdit::singleline(&mut self.fecha).desired_width(90.0));

                    ui.add_space(10.0);

                    ui.label("💲");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.amount)
                            .desired_width(80.0)
                            .hint_text("0.00"),
                    );
                });

                ui.add_space(4.0);

                egui::Grid::new("input_grid_compact")
                    .num_columns(2)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Categoría:");
                        egui::ComboBox::from_id_salt("cat_select_new")
                            .selected_text(
                                self.json_content
                                    .categorias
                                    .get(self.valor)
                                    .unwrap_or(&"?".to_string()),
                            )
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                for (val, key) in self.json_content.categorias.iter().enumerate() {
                                    ui.selectable_value(&mut self.valor, val, key);
                                }
                            });
                        ui.end_row();

                        ui.label("Nota:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.description)
                                .hint_text("Opcional")
                                .desired_width(180.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(RichText::new("💾 Guardar").strong()).clicked() {
                            self.process_new_record();
                        }
                    });
                });
            });
        });
    }

    fn process_new_record(&mut self) {
        if self.fecha.is_empty() {
            self.error = String::from("Falta la fecha");
            return;
        }
        match self.amount.parse::<f32>() {
            Ok(val) => {
                let tipo = if val < 0.0 {
                    TipoMovimiento::Gasto
                } else {
                    TipoMovimiento::Ingreso
                };
                self.json_content.transacciones.push(Movimiento::new(
                    self.fecha.clone(),
                    tipo,
                    self.description.clone(),
                    self.valor,
                    val.abs(),
                ));
                self.description = String::new();
                self.amount = String::new();
                self.error = String::new();
                self.update_categorias();
                self.get_points();
            }
            Err(_) => self.error = String::from("El monto debe ser numérico"),
        }
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        ui.heading("Distribución por Categorías");

        let available_size = ui.available_size();
        let height = available_size.y * 0.6;
        let width = available_size.x / 2.0; // Dividimos el ancho en 2

        ui.horizontal(|ui| {
            ui.allocate_ui(Vec2::new(width, available_size.y), |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Ingresos").strong().color(Color32::GREEN));
                    let (_, painter) =
                        ui.allocate_painter(Vec2::new(width * 0.9, height), Sense::hover());
                    let rect = painter.clip_rect();
                    let center = rect.center();
                    let radius = rect.height().min(rect.width()) / 2.5;

                    draw_donut(
                        &painter,
                        center,
                        radius,
                        &self.ingresos_cat,
                        self.ingresos_cat_tot,
                        &self.json_content.colores,
                    );

                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .id_salt("ingresos_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (cat_idx, monto) in &self.ingresos_cat {
                                let pct = if self.ingresos_cat_tot > 0.0 {
                                    (monto * 100.0) / self.ingresos_cat_tot
                                } else {
                                    0.0
                                };
                                ui.label(format!(
                                    "{}: {:.1}% (${})",
                                    self.json_content.categorias[*cat_idx], pct, monto
                                ));
                            }
                        });
                });
            });

            ui.allocate_ui(Vec2::new(width, available_size.y), |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Gastos").strong().color(Color32::RED));
                    let (_, painter) =
                        ui.allocate_painter(Vec2::new(width * 0.9, height), Sense::hover());
                    let rect = painter.clip_rect();
                    let center = rect.center();
                    let radius = rect.height().min(rect.width()) / 2.5;

                    draw_donut(
                        &painter,
                        center,
                        radius,
                        &self.gastos_cat,
                        self.gastos_cat_tot,
                        &self.json_content.colores,
                    );

                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .id_salt("gastos_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (cat_idx, monto) in &self.gastos_cat {
                                let pct = if self.gastos_cat_tot > 0.0 {
                                    (monto * 100.0) / self.gastos_cat_tot
                                } else {
                                    0.0
                                };
                                ui.label(format!(
                                    "{}: {:.1}% (${})",
                                    self.json_content.categorias[*cat_idx], pct, monto
                                ));
                            }
                        });
                });
            });
        });
    }

    fn grafica(&mut self, ui: &mut egui::Ui) {
        if self.ver_gra == GraficaVer::Grafica {
            ui.horizontal(|ui| {
                ui.heading("Evolución del Balance");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(format!("Máximo Flujo: {:.2}", self.max)).small());
                });
            });

            let mov_sort = self.mov_sort.clone();
            let formatter = move |x: GridMark, _: &RangeInclusive<f64>| -> String {
                if x.value >= 0.0 && (x.value as usize) < mov_sort.len() {
                    mov_sort[x.value as usize].clone()
                } else {
                    String::new()
                }
            };

            let plot = egui_plot::Plot::new("financial_plot")
                .show_x(false)
                .show_y(true)
                .clamp_grid(true)
                .auto_bounds(egui::Vec2b::TRUE)
                .x_axis_formatter(formatter)
                .legend(egui_plot::Legend::default());

            let p = PlotPoints::new(self.points.clone());
            let l = PlotPoints::new(self.lines.clone());

            plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new("Balance", l).width(2.0));
                plot_ui.points(
                    Points::new("Puntos", p)
                        .shape(MarkerShape::Circle)
                        .radius(5.0),
                );

                if plot_ui.response().clicked() {
                    let pp = plot_ui.pointer_coordinate();
                    if let Some(p) = pp {
                        let idx = p.x.round() as usize;
                        if idx < self.points.len() {
                            let val = self.points[idx][1];
                            if (p.y - val).abs() < (self.max * 0.1).max(1.0) {
                                self.ver_gra = GraficaVer::Elemento;
                                self.ver_gra_i = idx;
                            }
                        }
                    }
                }
            });
            ui.label(
                RichText::new("Haz clic en un punto para ver detalles del día.")
                    .weak()
                    .small(),
            );
        } else {
            if ui.button("⬅ Regresar a la Gráfica").clicked() {
                self.ver_gra = GraficaVer::Grafica;
            }
            ui.separator();

            if self.ver_gra_i < self.mov_sort.len() {
                let fecha_actual = &self.mov_sort[self.ver_gra_i];
                ui.heading(format!("Detalles: {}", fecha_actual));

                let mut daily_balance = 0.0;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for j in &self.json_content.transacciones {
                        if &j.fecha == fecha_actual {
                            let is_income = j.tipo == TipoMovimiento::Ingreso;
                            let color = if is_income {
                                Color32::GREEN
                            } else {
                                Color32::RED
                            };
                            let sign = if is_income { "+" } else { "-" };

                            if is_income {
                                daily_balance += j.monto;
                            } else {
                                daily_balance -= j.monto;
                            }

                            Frame::group(ui.style()).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&self.json_content.categorias[j.categoria])
                                            .strong(),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("{}{}", sign, j.monto))
                                                .color(color)
                                                .strong()
                                                .size(16.0),
                                        );
                                    });
                                });
                                ui.label(&j.description);
                            });
                            ui.add_space(5.0);
                        }
                    }
                });
                ui.separator();
                ui.label(
                    RichText::new(format!("Balance del día: {:.2}", daily_balance))
                        .strong()
                        .size(18.0),
                );
            }
        }
    }
}

fn draw_transaction_card(
    ui: &mut Ui,
    mov: &Movimiento,
    remove: &mut i32,
    idx: i32,
    is_income: bool,
    edit_state: &mut (i32, TipoMovimiento),
    colores: &[[f32; 3]],
) {
    let cat_color = array_to_color(
        colores
            .get(mov.categoria)
            .copied()
            .unwrap_or([0.5, 0.5, 0.5]),
    );
    let bg_color = faded(cat_color, ui);

    Frame::NONE
        .fill(bg_color)
        .corner_radius(6.0)
        .stroke(Stroke::new(1.0, cat_color.linear_multiply(0.3)))
        .inner_margin(6.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&mov.fecha).size(10.0).weak());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let amount_color = if is_income {
                        ui.visuals().selection.stroke.color
                    } else {
                        ui.visuals().error_fg_color
                    };
                    ui.label(
                        RichText::new(format!(
                            "{}{:.2}",
                            if is_income { "+" } else { "-" },
                            mov.monto
                        ))
                        .color(amount_color)
                        .strong(),
                    );
                });
            });

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 4.0, cat_color);

                ui.label(RichText::new(&mov.description).size(13.0));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("🗑").on_hover_text("Eliminar").clicked() {
                        *remove = idx;
                    }
                    if ui.small_button("✏").on_hover_text("Editar").clicked() {
                        *edit_state = (
                            idx,
                            if is_income {
                                TipoMovimiento::Ingreso
                            } else {
                                TipoMovimiento::Gasto
                            },
                        );
                    }
                });
            });
        });
    ui.add_space(4.0);
}

fn draw_edit_card(
    ui: &mut Ui,
    mov: &mut Movimiento,
    edit_state: &mut (i32, TipoMovimiento),
    cambiar: &mut bool,
    categorias: &[String],
) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.label(RichText::new("Editando...").weak().small());
        let mut edit_flag = false;
        edit_valor(ui, mov, edit_state, &mut edit_flag, categorias);
        if edit_flag {
            *cambiar = true;
        }
    });
    ui.add_space(4.0);
}

fn edit_valor(
    ui: &mut egui::Ui,
    mov: &mut Movimiento,
    edit: &mut (i32, TipoMovimiento),
    p: &mut bool,
    categorias_i: &[String],
) {
    ui.horizontal(|ui| {
        ui.label("📅");
        if ui
            .add(egui::TextEdit::singleline(&mut mov.fecha).desired_width(90.0))
            .changed()
        {
            *p = true;
        }

        ui.add_space(10.0);

        ui.label("💲");
        let mut g = format!("{}", mov.monto);
        if ui
            .add(egui::TextEdit::singleline(&mut g).desired_width(70.0))
            .changed()
        {
            if let Ok(result) = g.parse::<f32>() {
                mov.monto = result;
                *p = true;
            }
        }
    });

    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("edit_combo")
            .selected_text(categorias_i.get(mov.categoria).unwrap_or(&"?".to_string()))
            .width(100.0)
            .show_ui(ui, |ui| {
                for (val, key) in categorias_i.iter().enumerate() {
                    if ui.selectable_value(&mut mov.categoria, val, key).changed() {
                        *p = true;
                    }
                }
            });

        if ui
            .add(
                egui::TextEdit::singleline(&mut mov.description)
                    .hint_text("Desc")
                    .desired_width(100.0),
            )
            .changed()
        {
            *p = true;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("✅").on_hover_text("Guardar cambios").clicked() {
                *edit = (-1, TipoMovimiento::Null);
                *p = true;
            }
        });
    });
}

fn draw_donut(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    data: &HashMap<usize, f32>,
    total: f32,
    colors: &[[f32; 3]],
) {
    if total == 0.0 {
        painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::GRAY));
        painter.text(
            center,
            Align2::CENTER_CENTER,
            "Sin datos",
            FontId::default(),
            Color32::GRAY,
        );
        return;
    }

    let thickness = radius * 0.4;
    let mut start_angle = 0.0f32;

    for (cat_idx, value) in data {
        let fraction = value / total;
        let sweep_angle = fraction * std::f32::consts::TAU;
        let color = array_to_color(colors.get(*cat_idx).copied().unwrap_or([0.5, 0.5, 0.5]));

        let stroke = Stroke::new(thickness, color);
        let steps = (sweep_angle.abs() * 20.0).max(4.0) as usize;
        let mut points = Vec::with_capacity(steps);
        for i in 0..=steps {
            let angle = start_angle + (sweep_angle * i as f32 / steps as f32);
            points.push(center + Vec2::new(angle.cos(), angle.sin()) * (radius - thickness / 2.0));
        }

        painter.add(epaint::PathShape::line(points, stroke));
        start_angle += sweep_angle;
    }
}

fn array_to_color(arr: [f32; 3]) -> Color32 {
    let r = (255. * arr[0]) as u8;
    let g = (255. * arr[1]) as u8;
    let b = (255. * arr[2]) as u8;
    Color32::from_rgb(r, g, b)
}

fn faded(color: Color32, ui: &egui::Ui) -> Color32 {
    let dark_mode = ui.visuals().dark_mode;
    let bg = ui.visuals().window_fill();
    let t = if dark_mode { 0.15 } else { 0.2 };
    let r = (color.r() as f32 * t + bg.r() as f32 * (1.0 - t)) as u8;
    let g = (color.g() as f32 * t + bg.g() as f32 * (1.0 - t)) as u8;
    let b = (color.b() as f32 * t + bg.b() as f32 * (1.0 - t)) as u8;
    Color32::from_rgb(r, g, b)
}

use crate::main_area;
fn toggle_ui(ui: &mut egui::Ui, con: &mut main_area::Content,) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let on:bool;
    if *con == main_area::Content::View{
        on=true;
    }else{
        on=false;
    }

    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        if on{
            *con = main_area::Content::Edit;
        }else{
            *con = main_area::Content::View;
        }
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, on, ""));

    if  *con != main_area::Content::Graph{
        if ui.is_rect_visible(rect) {
            let how_on = ui.ctx().animate_bool(response.id, on);
            let visuals = ui.style().interact_selectable(&response, on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
            let center = egui::pos2(circle_x, rect.center().y);
            ui.painter()
                .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
        }

        response
    }else{
        if ui.button("Return").clicked(){
            *con = main_area::Content::View;
        }
        response
    }

}

pub fn toggle(con: &mut main_area::Content) -> impl egui::Widget + '_{
        move |ui: &mut egui::Ui| toggle_ui(ui, con)
}

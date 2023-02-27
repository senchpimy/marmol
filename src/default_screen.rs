use eframe::egui::CentralPanel;

pub fn default(ctx:&egui::Context){
            CentralPanel::default().show(ctx,|ui|{
                ui.heading("Marmol");
                ui.label("select a vault");
                ui.label("configuration");
            });
}

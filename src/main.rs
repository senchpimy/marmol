fn main() -> Result<(), eframe::Error> {
    let t0 = std::time::Instant::now();
    eprintln!("[TIMING] main start");
    #[cfg(not(target_os = "android"))]
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("swc_ecma_codegen", log::LevelFilter::Off)
        .filter_module("swc_ecma_transforms_base", log::LevelFilter::Off)
        .filter_module("swc", log::LevelFilter::Off)
        .filter_module("swc_common", log::LevelFilter::Off)
        .filter_module("swc_ecma_parser", log::LevelFilter::Off)
        .filter_module("tracing", log::LevelFilter::Off)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .try_init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eprintln!("[TIMING] before run_native: {:?}", t0.elapsed());
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|cc| {
            eprintln!("[TIMING] in run_native closure: {:?}", t0.elapsed());
            egui_extras::install_image_loaders(&cc.egui_ctx);
            eprintln!("[TIMING] after install_image_loaders: {:?}", t0.elapsed());
            Ok(Box::new(marmol::Marmol::new(cc)))
        }),
    )
}
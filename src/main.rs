use notepadppp::ui::NotepadApp;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let files: Vec<std::path::PathBuf> = std::env::args().skip(1).map(std::path::PathBuf::from).collect();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_maximized(true)
            .with_title("Notepad+++"),
        ..Default::default()
    };
    eframe::run_native(
        "Notepad+++",
        options,
        Box::new(move |cc| Ok(Box::new(NotepadApp::with_files(cc, files)))),
    )
}

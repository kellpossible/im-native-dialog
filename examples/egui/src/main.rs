use std::path::PathBuf;

use egui_glium::NativeOptions;
use epi::App;
use im_native_dialog::ImNativeFileDialog;

#[derive(Default)]
struct ExampleApp {
    file_path: PathBuf,
    file_path_dialog: ImNativeFileDialog<Option<PathBuf>>,
}

impl ExampleApp {
    fn new() -> Self {
        Self::default()
    }
}

impl App for ExampleApp {
    fn update(&mut self, ctx: &epi::egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        if let Some(result) = self.file_path_dialog.check() {
            match result {
                Ok(Some(path)) => self.file_path = path,
                Ok(None) => {}
                Err(error) => {
                    eprintln!("Error selecting xplane_path: {}", error)
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_enabled(!self.file_path_dialog.is_open());
                ui.label("File Path");
                let text_original = self.file_path.to_string_lossy().to_string();
                let mut text_edit = text_original.clone();
                ui.text_edit_singleline(&mut text_edit);
                if text_edit != text_original {
                    self.file_path = PathBuf::from(text_edit);
                }
                if ui.button("Browse").clicked() {
                    let location = self
                        .file_path
                        .parent()
                        .map(|location| location.to_path_buf());
                    self.file_path_dialog
                        .open_single_file(location)
                        .expect("Unable to open file_path dialog");
                }
            });
        });
    }

    fn name(&self) -> &str {
        "Example Path Select"
    }
}

fn main() {
    egui_glium::run(Box::new(ExampleApp::new()), NativeOptions::default())
}

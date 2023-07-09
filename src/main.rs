use std::path::Path;

use asyncgit::sync::RepoPath;

use eframe::{egui, epaint::Color32};
fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        transparent: true,
        ..Default::default()
    };
    eframe::run_native(
        "Git",
        options,
        Box::new(|_cc| Box::<GitViewer>::new(GitViewer::new())),
    )
    .unwrap();
}

struct GitViewer {}

impl GitViewer {
    pub fn new() -> Self {
        let repo_path =
            RepoPath::Path(Path::new("/Users/shuto/develop/github/sj/dearx").to_path_buf());
        // let branch_name = asyncgit::cached::BranchName::new(RefCell::new(repo_path.clone()));
        let _branch_infos: Vec<String> = match asyncgit::sync::get_branches_info(&repo_path, true) {
            Ok(_) => Vec::new(),
            Err(_) => Vec::new(),
        };
        Self {}
    }
}

impl eframe::App for GitViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::containers::Frame {
            fill: Color32::from_rgba_premultiplied(20, 20, 30, 20),
            ..Default::default()
        };

        egui::SidePanel::left("Branches")
            .frame(frame.clone())
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| ui.heading("Branches"));
                ui.separator();
                ui.vertical_centered(|ui| ui.heading("Item"));
            });
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| ui.button("My Button"));
    }
}

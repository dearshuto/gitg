use std::{path::Path, sync::Arc, sync::Mutex};

use asyncgit::sync::RepoPath;

use eframe::{egui, epaint::Color32};
use gitg::{GitStructureService, WatchTask};

#[tokio::main]
async fn main() {
    let service = GitStructureService::default();
    let service_shared = Arc::new(Mutex::new(service));

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        transparent: true,
        ..Default::default()
    };

    eframe::run_native(
        "Git",
        options,
        Box::new(|_cc| Box::new(GitViewer::new(service_shared))),
    )
    .unwrap();
}

struct GitViewer {
    #[allow(dead_code)]
    system: Arc<Mutex<GitStructureService<()>>>,
    task_list: Vec<WatchTask>,
}

impl GitViewer {
    pub fn new(service: Arc<Mutex<GitStructureService<()>>>) -> Self {
        let path = Path::new("/Users/shuto/develop/github/sj/gitg/src").to_path_buf();
        let task = service.lock().unwrap().watch(path);

        let repo_path = RepoPath::Path(Path::new("").to_path_buf());
        // let branch_name = asyncgit::cached::BranchName::new(RefCell::new(repo_path.clone()));
        let _branch_infos: Vec<String> = match asyncgit::sync::get_branches_info(&repo_path, true) {
            Ok(_) => Vec::new(),
            Err(_) => Vec::new(),
        };
        Self {
            system: service,
            task_list: vec![task],
        }
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

    fn on_close_event(&mut self) -> bool {
        self.system.lock().unwrap().unwatch_all();

        while let Some(task) = self.task_list.pop() {
            tokio::spawn(async move { task.kill().await });
        }
        true
    }
}

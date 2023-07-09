use std::{path::Path, sync::Arc, sync::Mutex};

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

        Self {
            system: service,
            task_list: vec![task],
        }
    }
}

impl eframe::App for GitViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let system = self.system.lock().unwrap();
        let frame = egui::containers::Frame {
            fill: Color32::from_rgba_premultiplied(20, 20, 30, 20),
            ..Default::default()
        };

        // ブランチ名を列挙
        egui::SidePanel::left("Branches")
            .frame(frame.clone())
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| ui.heading("Branches"));

                // 列挙するブランチ名はソートした方がいい？
                for branch_info in system.branch_infos() {
                    ui.separator();
                    ui.label(branch_info.name.to_string());
                }
            });

        // ここでツリーを表示したい
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            for commit_info in system.commit_infos() {
                ui.label(format!("{}: {}", commit_info.author, commit_info.message));
            }
        });
    }

    fn on_close_event(&mut self) -> bool {
        self.system.lock().unwrap().unwatch_all();

        while let Some(task) = self.task_list.pop() {
            tokio::spawn(async move { task.kill().await });
        }
        true
    }
}

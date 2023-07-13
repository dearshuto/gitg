use std::{path::Path, sync::Arc, sync::Mutex};

use eframe::{
    egui,
    egui::{Id, Painter},
    epaint::{Color32, Pos2, Stroke},
};
use gitg::{GitStructureService, ICommandBuffer, Plotter, WatchTask};

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
        let path = Path::new(".");
        let task = service.lock().unwrap().watch(path.to_path_buf());

        let repo_path = asyncgit::sync::RepoPath::Path(path.to_path_buf());
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
            let painter = ctx.layer_painter(egui::LayerId {
                order: egui::Order::Foreground,
                id: Id::new("aaa"),
            });

            let screen_rext = ui.max_rect();

            // コミット
            let mut index = 0;
            for commit_info in system.commit_infos() {
                ui.label(format!(
                    "{}: {}",
                    commit_info.author,
                    commit_info.id.get_short_string()
                ));

                painter.rect_filled(
                    eframe::epaint::Rect {
                        min: eframe::epaint::Pos2 {
                            x: screen_rext.min.x + 10.0,
                            y: 40.0 * index as f32,
                        },
                        max: eframe::epaint::Pos2 {
                            x: screen_rext.min.x + 15.0,
                            y: 40.0 * index as f32 + 5.0,
                        },
                    },
                    0.0,
                    Color32::RED,
                );

                index += 1;
            }

            // 接続
            let mut command_buffer = CommandBuffer::new(&painter);
            let plotter = Plotter::default();
            plotter.plot(&mut command_buffer);
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

struct CommandBuffer<'a> {
    painter: &'a Painter,
}

impl<'a> CommandBuffer<'a> {
    pub fn new(painter: &'a Painter) -> Self {
        Self { painter }
    }
}

impl<'a> ICommandBuffer for CommandBuffer<'a> {
    fn push_point(&mut self, x: f32, y: f32) {
        self.painter.rect_filled(
            eframe::epaint::Rect {
                min: eframe::epaint::Pos2 { x, y },
                max: eframe::epaint::Pos2 {
                    x: x + 5.0,
                    y: y + 5.0,
                },
            },
            0.0,
            Color32::KHAKI,
        );
    }

    fn push_line(&mut self, begin: [f32; 2], end: [f32; 2]) {
        let begin = Pos2 {
            x: begin[0],
            y: begin[1],
        };
        let end = Pos2 {
            x: end[0],
            y: end[1],
        };
        self.painter
            .line_segment([begin, end], Stroke::new(5.0, Color32::GREEN));
    }
}

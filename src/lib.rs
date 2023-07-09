use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

use asyncgit::sync::{BranchInfo, CommitInfo, RepoPath};
use notify::{Event, RecommendedWatcher, Watcher};
use tokio::task::JoinHandle;

pub struct WatchTask {
    #[allow(dead_code)]
    is_running: Arc<Mutex<bool>>,
    handle: JoinHandle<()>,

    // 終了時にブロッキングを解除するために使う
    tx: Sender<Result<Event, notify::Error>>,
}

impl WatchTask {
    pub fn new(path: PathBuf) -> Self {
        let is_running = Arc::new(Mutex::new(true));
        let is_running_binding = is_running.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        let tx_cache = tx.clone();
        let handle = tokio::spawn(async move {
            let mut watcher: RecommendedWatcher =
                Watcher::new_immediate(move |res| tx.send(res).unwrap()).unwrap();
            watcher
                .watch(path, notify::RecursiveMode::Recursive)
                .unwrap();

            for res in rx {
                if !is_running_binding.lock().unwrap().deref() {
                    break;
                }

                match res {
                    Ok(event) => {
                        // aaa
                        println!("{:?}", event)
                    }
                    Err(_) => {}
                }
            }
        });

        Self {
            is_running,
            handle,
            tx: tx_cache,
        }
    }

    pub async fn kill(self) {
        // フラグを消す
        *self.is_running.lock().unwrap() = false;

        // 適当にイベントを発生させる
        self.tx
            .send(Ok(Event::new(notify::EventKind::Any)))
            .unwrap();

        // 監視タスクが完了するのを待つ
        self.handle.await.unwrap()
    }
}

pub trait IGitStructureWatcher {
    fn on_changed(&mut self);
}

pub struct GitStructureService<TWatcher> {
    #[allow(dead_code)]
    watcher: TWatcher,
    branch_infos: Vec<BranchInfo>,
    commit_infos: Vec<CommitInfo>,
}

impl<T> GitStructureService<T> {
    pub fn branch_infos(&self) -> &[BranchInfo] {
        &self.branch_infos
    }

    pub fn commit_infos(&self) -> &[CommitInfo] {
        &self.commit_infos
    }
}

impl GitStructureService<()> {
    pub fn watch(&mut self, path: PathBuf) -> WatchTask {
        WatchTask::new(path)
    }

    pub fn unwatch_all(&mut self) {}
}

impl Default for GitStructureService<()> {
    fn default() -> Self {
        let path = Path::new(".").to_path_buf();
        let repo_path = RepoPath::Path(path.to_path_buf());

        let branch_infos = match asyncgit::sync::get_branches_info(&repo_path, true) {
            Ok(item) => item,
            Err(_) => Vec::new(),
        };

        let mut commit_infos = Vec::new();
        for branch_info in &branch_infos {
            let commit_info =
                asyncgit::sync::get_commit_info(&repo_path, &branch_info.top_commit).unwrap();
            commit_infos.push(commit_info);
        }

        Self {
            watcher: (),
            branch_infos,
            commit_infos,
        }
    }
}

impl<TWatcher: IGitStructureWatcher> GitStructureService<TWatcher> {
    pub fn new(watcher: TWatcher) -> Self {
        Self {
            watcher,
            branch_infos: Default::default(),
            commit_infos: Vec::default(),
        }
    }
}

impl<TWatcher: IGitStructureWatcher> GitStructureService<Arc<Mutex<TWatcher>>> {
    pub fn new_shared(watcher: Arc<Mutex<TWatcher>>) -> Self {
        Self {
            watcher,
            branch_infos: Default::default(),
            commit_infos: Vec::default(),
        }
    }
}

use std::{
    collections::HashSet,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

use asyncgit::sync::{BranchInfo, CommitInfo, RepoPath};
use eframe::epaint::ahash::HashMap;
use git2::Oid;
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

    // コミット全て
    id_table: HashSet<Oid>,

    // id の過去に向かう方向の親子関係
    hierarchy_table: HashMap<Oid, Vec<Oid>>,
}

impl<T> GitStructureService<T> {
    pub fn branch_infos(&self) -> &[BranchInfo] {
        &self.branch_infos
    }

    pub fn commit_infos(&self) -> &[CommitInfo] {
        &self.commit_infos
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

        // コミットを網羅
        let mut ids = HashSet::default();
        let mut hierarchy_table = HashMap::default();
        let repository = git2::Repository::open(&path).unwrap();
        for branch in repository.branches(None).unwrap() {
            let (branch, _type) = branch.unwrap();
            println!("{}", branch.name().unwrap().unwrap());

            let c = branch.get().peel_to_commit().unwrap();

            // すでに走査済みだったらなにもしない
            if ids.contains(&c.id()) {
                continue;
            }

            let mut parent_ids = Vec::new();
            for parent in c.parents() {
                parent_ids.push(parent.id());
                ids.insert(parent.id());
            }

            // ヒエラルキー情報を追加
            hierarchy_table.insert(c.id(), parent_ids);
        }

        let mut commit_ids = Vec::default();
        let count = asyncgit::sync::LogWalker::new(&repository, 100)
            .unwrap()
            .read(&mut commit_ids)
            .unwrap();

        let commit_infos =
            asyncgit::sync::get_commits_info(&repo_path, &commit_ids, count).unwrap();
        Self {
            watcher: (),
            branch_infos,
            commit_infos,
            id_table: ids,
            hierarchy_table,
        }
    }
}

impl<TWatcher: IGitStructureWatcher> GitStructureService<TWatcher> {
    pub fn new(watcher: TWatcher) -> Self {
        Self {
            watcher,
            branch_infos: Default::default(),
            commit_infos: Vec::default(),
            id_table: Default::default(),
            hierarchy_table: Default::default(),
        }
    }
}

impl<TWatcher: IGitStructureWatcher> GitStructureService<Arc<Mutex<TWatcher>>> {
    pub fn new_shared(watcher: Arc<Mutex<TWatcher>>) -> Self {
        Self {
            watcher,
            branch_infos: Default::default(),
            commit_infos: Vec::default(),
            id_table: Default::default(),
            hierarchy_table: Default::default(),
        }
    }
}

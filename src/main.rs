use std::{cell::RefCell, path::Path, sync::Arc};

use asyncgit::sync::RepoPath;

fn main() {
    let repo_path = RepoPath::Path(Path::new("/Users/shuto/develop/github/sj/dearx").to_path_buf());
    let mut branch_name = asyncgit::cached::BranchName::new(RefCell::new(repo_path.clone()));

    let branch_infos = asyncgit::sync::get_branches_info(&repo_path, true).unwrap();
    for info in &branch_infos {
        println!("name: {}", info.name);
        println!("    name: {}", info.reference);
        println!("    name: {}", info.top_commit_message);
        println!("    name: {:?}", info.top_commit);
    }
}

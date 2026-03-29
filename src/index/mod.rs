use std::collections::HashSet;

use crate::{file::PathStr, git::FileChange};

pub mod index;
pub mod tokenizer;
pub mod search;

#[derive(Debug, Clone, Default)]
pub struct IndexPlan {
    pub deletes: Vec<PathStr>,
    pub adds: Vec<PathStr>,
}

pub fn build_index_plan(file_changes: &[FileChange]) -> IndexPlan {
    let mut deletes = HashSet::<PathStr>::new();
    let mut adds = HashSet::<PathStr>::new();

    for ch in file_changes {
        match ch {
            FileChange::Added { path } => {
                adds.insert(path.clone());
            }
            FileChange::Deleted { path } => {
                deletes.insert(path.clone());
            }
            FileChange::Modified { path } => {
                deletes.insert(path.clone());
                adds.insert(path.clone());
            }
            FileChange::Renamed { old_path, new_path } => {
                deletes.insert(old_path.clone());
                adds.insert(new_path.clone());
            }
        }
    }

    let mut deletes: Vec<_> = deletes.into_iter().collect();
    let mut adds: Vec<_> = adds.into_iter().collect();

    deletes.sort_by(|a, b| path_depth(b).cmp(&path_depth(a)).then_with(|| b.cmp(a)));
    adds.sort_by(|a, b| path_depth(a).cmp(&path_depth(b)).then_with(|| a.cmp(b)));

    IndexPlan { deletes, adds }
}

fn path_depth(path: &PathStr) -> usize {
    path.split('/').filter(|s| !s.is_empty()).count()
}


use std::sync::Arc;

use chrono::Utc;

use crate::{
    git::{FileChange, GitService},
    scheduler::{BoxedTask, TaskID, TaskPriority, TaskScheduler},
    web::context::SiteContextShared,
};

pub fn init_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| {
        async move {
            log::info!("Initialization task running");

            // インデックスの初期化
            ctx.scheduler
                .push_task(
                    TaskID::BUILD_INDEX,
                    build_index_all_task(),
                    TaskPriority::HIGH,
                    None,
                    None,
                )
                .await;

            // gitリポジトリのクローンとインデックスの更新
            if ctx.config.git_config.enable_remote {
                ctx.scheduler
                    .push_task(
                        TaskID::GIT_UPDATE,
                        git_update_task(),
                        TaskPriority::HIGH,
                        None,
                        None,
                    )
                    .await;
            } else {
                log::info!("Remote Git repository is disabled, skipping clone/load");
            }

            // 初回スケジューリング
            let next_ready = TaskScheduler::next_5min(Utc::now());
            ctx.scheduler
                .push_task(
                    TaskID::CRON,
                    cron_task(),
                    TaskPriority::HIGH,
                    Some(next_ready),
                    None,
                )
                .await;

            log::info!("Initialization task finished");
        }
    })
}

pub fn build_index_all_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
        log::info!("Build index all task running");
        let timer = std::time::Instant::now();
        ctx.index.index_all(&ctx).await;
        let duration = timer.elapsed();
        log::info!("Build index all task finished in {:.2?}", duration);
    })
}

pub fn storage_save_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
        log::info!("Storage save task running");
        if let Err(e) = ctx
            .storage
            .save(ctx.config.storage_config.storage_file.as_ref())
        {
            log::error!("Failed to save storage: {}", e);
        } else {
            log::info!("Storage saved successfully");
        }
        log::info!("Storage save task finished");
    })
}

pub fn git_update_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
        log::info!("Git update task running");
        match GitService::update_async(ctx.config.clone()).await {
            Ok(update_outcome) => {
                if update_outcome.changed {
                    let mut system_info = ctx.system_info.load_full();
                    Arc::make_mut(&mut system_info).content_hash = update_outcome.new_commit.clone();
                    ctx.system_info.store(system_info);
                    log::info!("{}", update_outcome);
                    log::info!("Cache purging and index updating...");
                    for change in &update_outcome.file_changes {
                        match change {
                            FileChange::Modified { path }
                            | FileChange::Deleted { path }
                            | FileChange::Renamed { old_path: path, .. } => {
                                ctx.file_service.purge_cache_by_path(path).await;
                                log::debug!("Purged cache for path: {}", path);
                            }
                            _ => {}
                        }
                    }
                    log::info!("Cache purged");
                    ctx.index.update_index(&update_outcome.file_changes, &ctx).await;
                    log::info!("Index updated successfully");
                } else {
                    let mut system_info = ctx.system_info.load_full();
                    Arc::make_mut(&mut system_info).content_hash = update_outcome.new_commit.clone();
                    ctx.system_info.store(system_info);
                    log::info!("Git repository is already up to date");
                }
            }
            Err(e) => log::error!("Failed to update git repository: {}", e),
        }
        log::info!("Git update task finished");
    })
}

pub fn session_gc_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
        log::info!("Session GC task running");
        let skip = getrandom::u32().expect("getrandom u32") % 20;
        let sessions = ctx
            .storage
            .sessions
            .sessions
            .iter()
            .skip(skip as usize)
            .step_by(20)
            .map(|entry| *entry.key())
            .collect::<Vec<_>>();

        let count = sessions.into_iter().fold(0, |acc, k| {
            ctx.auth_manager.gc_sessions(&k);
            acc + 1
        });
        log::info!("Session GC task finished (removed {} sessions)", count);
    })
}

pub fn cron_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| {
        async move {
            log::info!("Cron task running");

            // 次回スケジューリング
            let next_ready = TaskScheduler::next_5min(Utc::now());
            let task_id = ctx
                .scheduler
                .push_task(
                    TaskID::CRON,
                    cron_task(),
                    TaskPriority::HIGH,
                    Some(next_ready),
                    None,
                )
                .await;

            // セッションGCタスクのスケジューリング
            ctx.scheduler
                .push_task(
                    TaskID::SESSION_GC,
                    session_gc_task(),
                    TaskPriority::LOW,
                    None,
                    None,
                )
                .await;

            // gitリポジトリの更新とインデックスの更新のスケジューリング
            if ctx.config.git_config.enable_remote {
                ctx.scheduler
                    .push_task(
                        TaskID::GIT_UPDATE,
                        git_update_task(),
                        TaskPriority::NORMAL,
                        None,
                        None,
                    )
                    .await;
            } else {
                log::info!("Remote Git repository is disabled, skipping clone/load");
            }

            // ストレージの保存
            if task_id.counter() % 12 == 0 {
                // 1時間に1回? スケジューラーの実装忘れた
                ctx.scheduler
                    .push_task(
                        TaskID::STORAGE_SAVE,
                        storage_save_task(),
                        TaskPriority::NORMAL,
                        None,
                        None,
                    )
                    .await;
            }

            log::info!("Cron task finished");
        }
    })
}



use std::sync::Arc;

use chrono::Utc;

use crate::{
    git::GitService,
    scheduler::{BoxedTask, TaskID, TaskPriority, TaskScheduler},
    web::context::SiteContextShared,
};

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

            let storage_save_task: BoxedTask =
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
                });

            let git_update_task: BoxedTask =
                TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
                    log::info!("Git update task running");
                    match GitService::update_async(ctx.config.clone()).await {
                        Ok(update_outcome) => {
                            if update_outcome.changed {
                                let mut system_info = ctx.system_info.load_full();
                                Arc::make_mut(&mut system_info).content_hash = update_outcome.new_commit.clone();
                                ctx.system_info.store(system_info);
                                ctx.index.update_index(&update_outcome.file_changes).await;
                                log::info!("{}", update_outcome);
                            } else {
                                log::info!("Git repository is already up to date");
                            }
                        }
                        Err(e) => log::error!("Failed to update git repository: {}", e),
                    }
                    log::info!("Git update task finished");
                });

            let session_gc_task: BoxedTask =
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
                });

            ctx.scheduler
                .push_task(
                    TaskID::SESSION_GC,
                    session_gc_task,
                    TaskPriority::LOW,
                    None,
                    None,
                )
                .await;

            if ctx.config.git_config.enable_remote {
                ctx.scheduler
                    .push_task(
                        TaskID::GIT_UPDATE,
                        git_update_task,
                        TaskPriority::NORMAL,
                        None,
                        None,
                    )
                    .await;
            } else {
                log::info!("Remote Git repository is disabled, skipping clone/load");
            }

            if task_id.counter() % 12 == 0 {
                // 1時間に1回? スケジューラーの実装忘れた
                ctx.scheduler
                    .push_task(
                        TaskID::STORAGE_SAVE,
                        storage_save_task,
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

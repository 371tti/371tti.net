use std::sync::Arc;

use chrono::Utc;
use log::info;

use crate::{
    scheduler::{BoxedTask, TaskID, TaskPriority, TaskScheduler},
    updater::UpdateService,
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

            // コンテンツ更新チェックタスクをスケジューリング
            let content_update_check_task: BoxedTask = TaskScheduler::boxed_task(
                move |ctx: Arc<SiteContextShared>| async move {
                    log::info!("Content update check task running");
                    let base_dir = std::path::PathBuf::from(ctx.config.base_dir.clone());
                    match UpdateService::update_content(
                        &base_dir,
                        &ctx.config.content_repo_url,
                        &ctx.config.content_repo_branch,
                        Some(&ctx.system_info.load_full().content_hash),
                    ) {
                        Ok(v) => {
                            info!(
                                "Content update check result: updated={}, previous_hash={:?}, current_hash={}",
                                v.updated, v.previous_hash, v.current_hash
                            );
                            if v.updated {
                                ctx.system_info
                                    .store(Arc::new(crate::web::context::SystemInfo {
                                        system_version: ctx
                                            .system_info
                                            .load()
                                            .system_version
                                            .clone(),
                                        content_hash: v.current_hash.clone(),
                                    }));
                            }
                        }
                        Err(e) => log::error!("Content update failed: {}", e),
                    }
                    log::info!("Content update check task finished");
                },
            );

            let storage_save_task: BoxedTask =
                TaskScheduler::boxed_task(move |ctx: Arc<SiteContextShared>| async move {
                    log::info!("Storage save task running");
                    if let Err(e) = ctx.storage.save(ctx.config.storage_file.as_ref()) {
                        log::error!("Failed to save storage: {}", e);
                    } else {
                        log::info!("Storage saved successfully");
                    }
                    log::info!("Storage save task finished");
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

            if ctx.config.auto_content_update {
                ctx.scheduler
                    .push_task(
                        TaskID::UPDATE_CHECK,
                        content_update_check_task,
                        TaskPriority::NORMAL,
                        None,
                        None,
                    )
                    .await;
            }

            ctx.scheduler
                .push_task(
                    TaskID::SESSION_GC,
                    session_gc_task,
                    TaskPriority::LOW,
                    None,
                    None,
                )
                .await;

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

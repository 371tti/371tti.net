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
            ctx.scheduler
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

            ctx.scheduler
                .push_task(
                    TaskID::UPDATE_CHECK,
                    content_update_check_task,
                    TaskPriority::NORMAL,
                    None,
                    None,
                )
                .await;

            log::info!("Cron task finished");
        }
    })
}

use chrono::Utc;

use crate::{context::SiteContext, task::scheduler::{BoxedTask, TaskID, TaskPriority, TaskScheduler}};

pub mod scheduler;

// 自分で自分をスケジュールするやつ
pub fn cron_task() -> BoxedTask {
    TaskScheduler::boxed_task(move |ctx: SiteContext| {
        async move {
            log::info!("Cron task running");

            // 次回スケジューリング
            let next_ready = TaskScheduler::next_5min(Utc::now());
            ctx.scheduler.push_task(
                TaskID::CRON,
                cron_task(),
                TaskPriority::HIGH,
                Some(next_ready),
                None,
            ).await;

            // セッションGCタスク
            let session_gc_task: BoxedTask = TaskScheduler::boxed_task(move |ctx: SiteContext| {
                async move {
                    log::info!("Session GC task running");
                    ctx.auth.session_gc_task().await;
                    log::info!("Session GC task finished");
                }
            });

            // ヘルスチェック
            let health_check_task: BoxedTask = TaskScheduler::boxed_task(move |ctx: SiteContext| {
                async move {
                    log::info!("Health check task running");
                    ctx.health.update("").await;
                    log::info!("Health check task finished");
                }
            });
            // etc...
            // 他の定期タスクもここに追加していく

            ctx.scheduler.push_task(
                TaskID::SESSION_GC,
                session_gc_task,
                TaskPriority::IDLE,
                None,
                None,
            ).await;

            ctx.scheduler.push_task(
                TaskID::HEALTH_CHECK,
                health_check_task,
                TaskPriority::HIGH,
                None,
                None,
            ).await;

            log::info!("Cron task finished");
        }
    })
}
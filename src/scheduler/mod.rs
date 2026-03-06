use std::{
    cmp::Ordering,
    collections::BTreeSet,
    fmt::Debug,
    pin::Pin,
    sync::{
        Arc,
        atomic::{self, AtomicU64},
    },
    time::Duration,
};

use chrono::{DateTime, Timelike, Utc};
use tokio::sync::{Notify, RwLock};

use crate::web::context::SiteContextShared;
pub mod task;

/// タスクの優先度
/// 0~255の範囲で指定
#[derive(Debug, Clone, Copy)]
pub struct TaskPriority(u8);

impl TaskPriority {
    pub const INSTANT: Self = Self(u8::MAX);
    pub const HIGH: Self = Self(192);
    pub const NORMAL: Self = Self(128);
    pub const LOW: Self = Self(64);
    pub const IDLE: Self = Self(0);
}

impl TaskPriority {
    pub fn new(priority: u8) -> Self {
        Self(priority)
    }
}

impl Ord for TaskPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for TaskPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TaskPriority {}
impl PartialEq for TaskPriority {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// タスクID
/// 上位16bit: prefix, 下位48bit: カウンタ
#[derive(Clone, Copy)]
pub struct TaskID(u64);

impl TaskID {
    pub const ANY: Self = Self(0);
    pub const CRON: Self = Self(1 << 48);
    pub const SESSION_GC: Self = Self(2 << 48);
    pub const HEALTH_CHECK: Self = Self(3 << 48);
    pub const ACCOUNT_SAVE: Self = Self(4 << 48);
    pub const GIT_UPDATE: Self = Self(5 << 48);
    pub const STORAGE_SAVE: Self = Self(6 << 48);

    pub fn new(prefix: u16, counter: u64) -> Self {
        let counter = counter & 0x0000FFFFFFFFFFFF;
        Self(((prefix as u64) << 48) | counter)
    }

    pub fn set_prefix(&mut self, prefix: u16) {
        let counter = self.0 & 0x0000FFFFFFFFFFFF;
        self.0 = ((prefix as u64) << 48) | counter;
    }

    pub fn set_counter(&mut self, counter: u64) {
        let counter = counter & 0x0000FFFFFFFFFFFF;
        let prefix = self.0 & 0xFFFF000000000000;
        self.0 = prefix | counter;
    }

    pub fn counter(&self) -> u64 {
        self.0 & 0x0000FFFFFFFFFFFF
    }

    pub fn prefix(&self) -> &'static str {
        match (self.0 >> 48) as u16 {
            0 => "ANY",
            1 => "CRON",
            2 => "SESSION_GC",
            3 => "HLTHCK",
            4 => "ACCTSV",
            5 => "GIT_UPDATE",
            6 => "STORAGE_SAVE",
            _ => "UNKNOWN",
        }
    }
}

impl Debug for TaskID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

impl Ord for TaskID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for TaskID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TaskID {}
impl PartialEq for TaskID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// タイマー待機中タスクのアイテム
pub struct WaitTaskItem {
    /// 一意
    id: TaskID,
    /// 優先度
    priority: TaskPriority,
    ready_at: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    task: BoxedTask,
}

impl Ord for WaitTaskItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // ready_at が早い順
        match self.ready_at.cmp(&other.ready_at) {
            Ordering::Equal => {
                // id のカウンタが小さい順
                self.id.counter().cmp(&other.id.counter())
            }
            other => other,
        }
    }
}

impl PartialOrd for WaitTaskItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for WaitTaskItem {}
impl PartialEq for WaitTaskItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// 実行待機中タスクのアイテム
pub struct ReadyTaskItem {
    /// 一意
    pub id: TaskID,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub task: BoxedTask,
}

impl From<WaitTaskItem> for ReadyTaskItem {
    fn from(item: WaitTaskItem) -> Self {
        Self {
            id: item.id,
            priority: item.priority,
            deadline: item.deadline,
            task: item.task,
        }
    }
}

impl Ord for ReadyTaskItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // priority が高い順
        match self.priority.cmp(&other.priority).reverse() {
            Ordering::Equal => {
                // deadline が早い順
                match (self.deadline, other.deadline) {
                    (Some(d1), Some(d2)) => d1.cmp(&d2),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => {
                        // id のカウンタが小さい順
                        self.id.counter().cmp(&other.id.counter())
                    }
                }
            }
            other => other,
        }
    }
}

impl PartialOrd for ReadyTaskItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for ReadyTaskItem {}
impl PartialEq for ReadyTaskItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct TaskScheduler {
    /// wait_queue
    /// 待機中タスクのキュー
    wait_queue: RwLock<BTreeSet<WaitTaskItem>>,
    /// wakeup_notify
    /// ウェイクアップ通知
    /// あたらしいタスクが追加されたときに通知する
    wakeup_notify: Notify,
    /// ready_queue
    /// 実行待機中タスクのキュー
    ready_queue: RwLock<BTreeSet<ReadyTaskItem>>,
    /// ready_notify
    /// 実行待機中タスクが追加されたときに通知する
    ready_notify: Notify,
    /// task_id_counter
    /// タスクIDのカウンタ
    task_id_counter: AtomicU64,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            wait_queue: RwLock::new(BTreeSet::new()),
            wakeup_notify: Notify::new(),
            ready_queue: RwLock::new(BTreeSet::new()),
            ready_notify: Notify::new(),
            task_id_counter: AtomicU64::new(0),
        }
    }

    async fn next_wakeup_time(&self) -> Option<DateTime<Utc>> {
        self.wait_queue
            .read()
            .await
            .iter()
            .next()
            .map(|item| item.ready_at)
    }

    pub async fn push_task(
        &self,
        mut id: TaskID,
        task: BoxedTask,
        priority: TaskPriority,
        ready_at: Option<DateTime<Utc>>,
        deadline: Option<DateTime<Utc>>,
    ) -> TaskID {
        let counter = self.task_id_counter.fetch_add(1, atomic::Ordering::SeqCst);
        id.set_counter(counter);

        if let Some(ready_at) = ready_at {
            let wait_item = WaitTaskItem {
                id,
                priority,
                ready_at,
                deadline,
                task,
            };
            self.wait_queue.write().await.insert(wait_item);
            self.wakeup_notify.notify_one();
        } else {
            let ready_item = ReadyTaskItem {
                id,
                priority,
                deadline,
                task,
            };
            self.ready_queue.write().await.insert(ready_item);
            self.wakeup_notify.notify_one();
        }
        id
    }

    async fn timer_loop(&self) {
        loop {
            let next_wakeup = self.next_wakeup_time().await;
            if let Some(wakeup_time) = next_wakeup {
                let now = Utc::now();
                if wakeup_time <= now {
                    self.ready_queue
                        .write()
                        .await
                        .insert(self.wait_queue.write().await.pop_first().unwrap().into());
                    self.ready_notify.notify_one();
                    continue;
                } else {
                    let delta = wakeup_time - now;
                    let duration = Duration::from_millis(delta.num_milliseconds().max(0) as u64);
                    tokio::select! {
                        _ = tokio::time::sleep(duration) => {},
                        _ = self.wakeup_notify.notified() => {},
                    }
                }
            } else {
                // タスクがない場合は通知を待つ
                self.wakeup_notify.notified().await;
            }
        }
    }

    async fn fetch_ready_task(&self) -> Option<ReadyTaskItem> {
        self.ready_queue.write().await.pop_first()
    }

    async fn fetch_ready_task_wait(&self) -> ReadyTaskItem {
        loop {
            if let Some(task) = self.fetch_ready_task().await {
                return task;
            } else {
                self.ready_notify.notified().await;
            }
        }
    }

    async fn execute_loop(&self, context: Arc<SiteContextShared>) {
        loop {
            let task_item = self.fetch_ready_task_wait().await;
            if task_item.deadline.map(|d| d < Utc::now()).unwrap_or(false) {
                log::warn!(
                    "Task {} 0x{:?} deadline exceeded, skipping execution",
                    task_item.id.prefix(),
                    task_item.id
                );
                continue;
            }
            log::debug!(
                "Executing task {} 0x{:?}",
                task_item.id.prefix(),
                task_item.id
            );
            (task_item.task)(context.clone()).await;
        }
    }

    pub async fn start(context: Arc<SiteContextShared>, worker_num: usize) {
        log::info!("Starting TaskScheduler with {} workers", worker_num);
        let timer_self = context.clone();
        tokio::spawn(async move {
            timer_self.scheduler.timer_loop().await;
        });
        log::info!("TaskScheduler timer loop started");

        for _ in 0..worker_num {
            let exec_self = context.clone();
            let exec_context = context.clone();
            tokio::spawn(async move {
                exec_self.scheduler.execute_loop(exec_context).await;
            });
        }
        log::info!("TaskScheduler {} worker(s) started", worker_num);
        log::info!("TaskScheduler started successfully");
    }

    /// 非同期タスクをBox化するユーティリティ関数
    pub fn boxed_task<F, Fut>(f: F) -> BoxedTask
    where
        F: Fn(Arc<SiteContextShared>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        Box::new(move |ctx| Box::pin(f(ctx)))
    }

    /// 次の5分間隔の時刻を取得
    pub fn next_5min(dt: DateTime<Utc>) -> DateTime<Utc> {
        let minute = dt.minute() as i64;
        // 次の5分刻みまでの追加分（常に正）
        let add = ((minute / 5) + 1) * 5 - minute;
        let next = dt + chrono::Duration::minutes(add);
        next.with_second(0).unwrap().with_nanosecond(0).unwrap()
    }

    pub fn next_min(dt: DateTime<Utc>) -> DateTime<Utc> {
        let next = dt + chrono::Duration::minutes(1);
        next.with_second(0).unwrap().with_nanosecond(0).unwrap()
    }
}

/// 非同期タスクをbox化
pub type BoxedTask =
    Box<dyn Fn(Arc<SiteContextShared>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

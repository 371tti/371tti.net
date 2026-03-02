use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Counter {
    pub total_req_count: AtomicU64,
    pub status_counters: StatusCounters,
    pub not_found_count: AtomicU64,
    pub unique_visitors: AtomicU64,
    pub page_view_count: AtomicU64,
}

#[derive(Serialize, Deserialize)]
pub struct StatusCounters {
    pub s1xx: AtomicU64,
    pub s2xx: AtomicU64,
    pub s3xx: AtomicU64,
    pub s4xx: AtomicU64,
    pub s5xx: AtomicU64,
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self {
            total_req_count: AtomicU64::new(0),
            status_counters: StatusCounters {
                s1xx: AtomicU64::new(0),
                s2xx: AtomicU64::new(0),
                s3xx: AtomicU64::new(0),
                s4xx: AtomicU64::new(0),
                s5xx: AtomicU64::new(0),
            },
            not_found_count: AtomicU64::new(0),
            unique_visitors: AtomicU64::new(0),
            page_view_count: AtomicU64::new(0),
        }
    }

    pub fn increment(&self, status_code: u16) {
        self.total_req_count.fetch_add(1, Ordering::Relaxed);
        match status_code {
            100..=199 => {
                self.status_counters.s1xx.fetch_add(1, Ordering::Relaxed);
            }
            200..=299 => {
                self.status_counters.s2xx.fetch_add(1, Ordering::Relaxed);
            }
            300..=399 => {
                self.status_counters.s3xx.fetch_add(1, Ordering::Relaxed);
            }
            400..=499 => {
                self.status_counters.s4xx.fetch_add(1, Ordering::Relaxed);
                if status_code == 404 {
                    self.not_found_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            500..=599 => {
                self.status_counters.s5xx.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn increment_unique_visitors(&self) -> u64 {
        self.unique_visitors.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn increment_page_views(&self) -> u64 {
        self.page_view_count.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn get_total(&self) -> u64 {
        self.total_req_count.load(Ordering::Relaxed)
    }

    pub fn get_status_counts(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.status_counters.s1xx.load(Ordering::Relaxed),
            self.status_counters.s2xx.load(Ordering::Relaxed),
            self.status_counters.s3xx.load(Ordering::Relaxed),
            self.status_counters.s4xx.load(Ordering::Relaxed),
            self.status_counters.s5xx.load(Ordering::Relaxed),
        )
    }

    pub fn text_report(&self) -> String {
        format!(
            "uu={} pv={} total={}",
            self.unique_visitors.load(Ordering::Relaxed),
            self.page_view_count.load(Ordering::Relaxed),
            self.total_req_count.load(Ordering::Relaxed),
        )
    }
}

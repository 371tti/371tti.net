use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use argon2::{Argon2, Params, Algorithm, Version};
use base64::{Engine, engine::general_purpose};
use log::{info, debug};

/// Target duration for benchmark in milliseconds
const TARGET_DURATION_MS: u64 = 200;

/// configuration for password hashing
#[derive(Serialize, Deserialize, Clone)]
pub struct HashConfig {
    pub pepper: String,
    pub memory_kib: u32,
    pub time_cost: u32,
    pub lanes: u32,
}

impl HashConfig {
    pub fn benchmark() -> Self {
        info!("Benchmarking HashConfig parameters...");
        let test_password = "benchmark_password";
        let salt = [0u8; 16]; // Fixed salt for benchmark
        let target_duration = Duration::from_millis(TARGET_DURATION_MS);

        info!("Benchmark assumptions: target_duration={:?}, test_password='{}', salt={:?}", target_duration, test_password, salt);

        let pepper = Self::generate_random_pepper();
        info!("Generated random pepper for benchmark");

        // Binary search for best memory
        let best_memory = Self::binary_search_param(target_duration, |memory| {
            let params = Params::new(memory, 3, 1, Some(32)).unwrap();
            let hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let start = Instant::now();
            let mut out = [0u8; 32];
            let mut adv = Vec::new();
            adv.extend_from_slice(&salt);
            let pepper_bytes = general_purpose::STANDARD.decode(&pepper).unwrap();
            adv.extend_from_slice(&pepper_bytes);
            hasher.hash_password_into(test_password.as_bytes(), &adv, &mut out).unwrap();
            let duration = start.elapsed();
            debug!("Memory {} KiB: duration={:?}", memory, duration);
            duration
        }, 32768, 1048576); // 32 MiB to 1024 MiB

        // Binary search for best time
        let best_time = Self::binary_search_param(target_duration, |time| {
            let params = Params::new(best_memory, time, 1, Some(32)).unwrap();
            let hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let start = Instant::now();
            let mut out = [0u8; 32];
            let mut adv = Vec::new();
            adv.extend_from_slice(&salt);
            let pepper_bytes = general_purpose::STANDARD.decode(&pepper).unwrap();
            adv.extend_from_slice(&pepper_bytes);
            hasher.hash_password_into(test_password.as_bytes(), &adv, &mut out).unwrap();
            let duration = start.elapsed();
            debug!("Time {}: duration={:?}", time, duration);
            duration
        }, 1, 10);

        // Binary search for best lanes
        let best_lanes = Self::binary_search_param(target_duration, |lanes| {
            let params = Params::new(best_memory, best_time, lanes, Some(32)).unwrap();
            let hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let start = Instant::now();
            let mut out = [0u8; 32];
            let mut adv = Vec::new();
            adv.extend_from_slice(&salt);
            let pepper_bytes = general_purpose::STANDARD.decode(&pepper).unwrap();
            adv.extend_from_slice(&pepper_bytes);
            hasher.hash_password_into(test_password.as_bytes(), &adv, &mut out).unwrap();
            let duration = start.elapsed();
            debug!("Lanes {}: duration={:?}", lanes, duration);
            duration
        }, 1, 8);

        let best_config = Self {
            pepper,
            memory_kib: best_memory,
            time_cost: best_time,
            lanes: best_lanes,
        };

        // Measure final duration
        let params = Params::new(best_memory, best_time, best_lanes, Some(32)).unwrap();
        let hasher = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let start = Instant::now();
        let mut out = [0u8; 32];
        let mut adv = Vec::new();
        adv.extend_from_slice(&salt);
        let pepper_bytes = general_purpose::STANDARD.decode(&best_config.pepper).unwrap();
        adv.extend_from_slice(&pepper_bytes);
        hasher.hash_password_into(test_password.as_bytes(), &adv, &mut out).unwrap();
        let final_duration = start.elapsed();

        info!("Best HashConfig: memory={} KiB, time={}, lanes={}, duration={:?}", best_config.memory_kib, best_config.time_cost, best_config.lanes, final_duration);
        best_config
    }

    fn generate_random_pepper() -> String {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("generate random pepper");
        general_purpose::STANDARD.encode(&bytes)
    }

    fn binary_search_param<F>(target: Duration, measure: F, min: u32, max: u32) -> u32
    where
        F: Fn(u32) -> Duration,
    {
        let mut low = min;
        let mut high = max;
        let mut best = min;
        let mut best_diff = Duration::from_secs(1000);

        while low <= high {
            let mid = low + (high - low) / 2;
            let duration = measure(mid);
            let diff = if duration > target {
                duration - target
            } else {
                target - duration
            };

            if diff < best_diff {
                best = mid;
                best_diff = diff;
            }

            if duration < target {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        best
    }
}

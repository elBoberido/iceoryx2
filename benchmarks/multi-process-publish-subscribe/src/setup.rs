use iceoryx2::prelude::*;

pub const MAX_ITERATIONS: usize = 10_000_000;

pub const LEADER_READY_EVENT_ID: EventId = EventId::new(1);
pub const FOLLOWER_READY_EVENT_ID: EventId = EventId::new(2);

pub const READY_EVENT_NAME: &'static str = "bench/event/ready";

pub const SETTINGS_SERVICE_NAME: &'static str = "bench/settings";
pub const SETTINGS_EVENT_NAME: &'static str = "bench/event/settings";

pub const LEADER_SERVICE_NAME: &'static str = "bench/leader";
pub const FOLLOWER_SERVICE_NAME: &'static str = "bench/follower";

pub const LATENCY_SERVICE_NAME: &'static str = "bench/latency";

use core::mem::MaybeUninit;

#[derive(Debug)]
pub struct SettingsTopic {
    pub iterations: usize,
}

#[derive(Debug)]
pub struct Info {
    pub timestamp: u128,
    pub warmup: bool,
    pub last: bool,
}

#[derive(Debug)]
pub struct BenchTopic<const N: usize> {
    pub info: Info,
    pub data: MaybeUninit<[u8; N]>,
}

#[derive(Debug)]
pub struct LatencyTopic {
    pub used_size: usize,
    pub latencies: [u64; MAX_ITERATIONS],
}

impl Default for LatencyTopic {
    fn default() -> Self {
        Self {
            used_size: 0,
            latencies: [0; MAX_ITERATIONS],
        }
    }
}

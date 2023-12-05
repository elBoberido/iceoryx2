mod follower;
mod leader;
mod params;
mod setup;

use crate::setup::*;

use iceoryx2::prelude::*;
use iceoryx2_bb_log::set_log_level;

use clap::Parser;

use std::process::Command;
use std::time::{Duration, SystemTime};

fn run_main_process(iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    if iterations > MAX_ITERATIONS {
        Err(format!("Exceeding max iterations of: {}", MAX_ITERATIONS))?;
    }

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    // ready setup
    let ready_event = node.service_builder(&ServiceName::new(READY_EVENT_NAME)?)
        .event()
        .open_or_create()?;

    let ready_listener = ready_event.listener_builder().create()?;

    // settings setup
    let settings_service = node.service_builder(&ServiceName::new(SETTINGS_SERVICE_NAME)?)
        .publish_subscribe::<SettingsTopic>()
        .max_publishers(1)
        .max_subscribers(2)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let settings_publisher = settings_service.publisher_builder().create()?;

    let settings_event = node.service_builder(&ServiceName::new(SETTINGS_EVENT_NAME)?)
        .event()
        .open_or_create()?;

    let settings_notifier = settings_event.notifier_builder().create()?;

    // latency result setup
    let latency_service = node.service_builder(&ServiceName::new(LATENCY_SERVICE_NAME)?)
        .publish_subscribe::<LatencyTopic>()
        .max_publishers(2)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(2)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let latency_subscriber = latency_service.subscriber_builder().create()?;

    println!("Spawning 'leader' and 'follower' process ...");
    let process_path = std::env::current_exe()?;
    let mut follower_process = Command::new(&process_path).arg("--follower").spawn()?;
    let mut leader_process = Command::new(&process_path).arg("--leader").spawn()?;
    println!("... done");

    // wait for leader and follower process to be ready
    let mut leader_ready = false;
    let mut follower_ready = false;
    let wait_for_ready_start_time = SystemTime::now();
    loop {
        match ready_listener.timed_wait_one(Duration::from_secs(1)) {
            Ok(Some(id)) => match id {
                LEADER_READY_EVENT_ID => {
                    leader_ready = true;
                }
                FOLLOWER_READY_EVENT_ID => {
                    follower_ready = true;
                }
                _ => Err(format!("Received invalid event id: {:?}", id))?,
            },
            _ => {}
        }

        if (leader_ready && follower_ready)
            || wait_for_ready_start_time.elapsed()? > Duration::from_secs(20)
        {
            break;
        }
    }

    if !leader_ready || !follower_ready {
        let _ = follower_process.kill();
        let _ = leader_process.kill();
        if !leader_ready && !follower_ready {
            Err("The leader and follower processes are not ready!")?;
        } else if !leader_ready {
            Err("The leader process is not ready!")?;
        } else {
            Err("The follower process is not ready!")?;
        }
    }

    settings_publisher.send_copy(SettingsTopic { iterations })?;
    settings_notifier.notify()?;

    println!("Waiting for benchmark to finish ...");

    let mut samples: Vec<_> = Vec::new();
    while samples.len() < 2 {
        if let Ok(Some(sample)) = latency_subscriber.receive() {
            samples.push(sample);
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    follower_process.wait()?;
    leader_process.wait()?;

    println!("... finished");

    println!("Processing ...");

    // let sample1 = latency_subscriber.receive()?.unwrap();
    // let sample2 = latency_subscriber.receive()?.unwrap();
    let mut total_time = 0;
    let mut total_iterations = 0;
    let mut max_latency = 0;
    let mut min_latency = 1_000_000_000;
    let mut max_latency_index = 0;
    let mut latency_larger_than_10us_count = 0;
    let mut latency_larger_than_100us_count = 0;
    let mut latency_larger_than_1000us_count = 0;
    for sample in samples {
        for i in 0..sample.used_size {
            let latency = sample.latencies[i];
            total_time += latency;
            if latency < min_latency {
                min_latency = latency;
            }
            if latency > max_latency {
                max_latency = latency;
                max_latency_index = i;
            }
            if latency > 10_000 {
                latency_larger_than_10us_count += 1;
            }
            if latency > 100_000 {
                latency_larger_than_100us_count += 1;
            }
            if latency > 1_000_000 {
                latency_larger_than_1000us_count += 1;
            }
        }
        total_iterations += sample.used_size;
    }

    println!("total transmissions: {}", total_iterations);
    println!("min latency: {}ns", min_latency);
    println!("avg latency: {}ns", total_time / (total_iterations as u64));
    println!(
        "max latency: {}ns at index: {}",
        max_latency, max_latency_index
    );
    println!(
        "latency larger than 10us count: {}",
        latency_larger_than_10us_count
    );
    println!(
        "latency larger than 100us count: {}",
        latency_larger_than_100us_count
    );
    println!(
        "latency larger than 1000us count: {}",
        latency_larger_than_1000us_count
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = params::Params::parse();

    set_log_level(iceoryx2_bb_log::LogLevel::Info);

    if params.leader {
        leader::run_leader_process()?;
    } else if params.follower {
        follower::run_follower_process()?;
    } else {
        run_main_process(params.iterations)?;
    }

    Ok(())
}

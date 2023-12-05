use crate::setup::*;

use iceoryx2::prelude::*;

use core::mem::MaybeUninit;
use core::time::Duration;
use std::time::SystemTime;

pub fn run_leader_process() -> Result<(), Box<dyn std::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    // settings setup
    let settings_service = node.service_builder(&ServiceName::new(SETTINGS_SERVICE_NAME)?)
        .publish_subscribe::<SettingsTopic>()
        .max_publishers(1)
        .max_subscribers(2)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let settings_subscriber = settings_service.subscriber_builder().create()?;

    let settings_event = node.service_builder(&ServiceName::new(SETTINGS_EVENT_NAME)?)
        .event()
        .open_or_create()?;

    let mut settings_listener = settings_event.listener_builder().create()?;

    // leader setup
    let leader_service = node.service_builder(&ServiceName::new(LEADER_SERVICE_NAME)?)
        .publish_subscribe::<BenchTopic<1024>>()
        .max_publishers(1)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let leader_publisher = leader_service.publisher_builder().create()?;

    // follower setup
    let follower_service = node.service_builder(&ServiceName::new(FOLLOWER_SERVICE_NAME)?)
        .publish_subscribe::<BenchTopic<1024>>()
        .max_publishers(1)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(1)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let follower_subscriber = follower_service.subscriber_builder().create()?;

    // latency result setup
    let latency_service = node.service_builder(&ServiceName::new(LATENCY_SERVICE_NAME)?)
        .publish_subscribe::<LatencyTopic>()
        .max_publishers(2)
        .max_subscribers(1)
        .history_size(0)
        .subscriber_max_buffer_size(2)
        .enable_safe_overflow(false)
        .open_or_create()?;

    let latency_publisher = latency_service.publisher_builder().create()?;
    let mut latency_sample = latency_publisher.loan()?;

    // ready setup
    let ready_event = node.service_builder(&ServiceName::new(READY_EVENT_NAME)?)
        .event()
        .open_or_create()?;

    let ready_notifier = ready_event.notifier_builder().create()?;

    // signal ready to main process
    ready_notifier.notify_with_custom_event_id(LEADER_READY_EVENT_ID)?;

    // wait for settings
    match settings_listener.timed_wait_one(Duration::from_secs(2)) {
        Ok(_) => { /* nothing to do */ }
        Err(e) => Err(format!("Error while waiting for settings: {:?}", e))?,
    }

    let settings = settings_subscriber.receive().unwrap().unwrap();

    let mut remaining = settings.iterations;
    let mut i = 0;
    let mut warmup = 10_000;
    loop {
        let remaining_next = remaining - 1;
        let send_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let sample = leader_publisher.loan_uninit()?.write_payload(BenchTopic {
            info: Info {
                timestamp: send_timestamp,
                warmup: warmup > 0,
                last: remaining_next == 0,
            },
            data: MaybeUninit::uninit(),
        });
        sample.send()?;

        let mut abort_counter = 100_000_000;
        let sample = loop {
            match follower_subscriber.receive() {
                Ok(None) => { /* nothing to do */ }
                Ok(Some(sample)) => {
                    break sample;
                }
                Err(e) => Err(format!("Error at receiving samples: {:?}", e))?,
            }
            abort_counter -= 1;
            if abort_counter == 0 {
                Err("The follower process is not responding")?;
            }
        };
        if warmup > 0 {
            warmup -= 1;
        } else {
            let receive_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos();
            let latency = receive_timestamp.saturating_sub(sample.info.timestamp);
            latency_sample.payload_mut().latencies[i] = latency as u64;
            remaining = remaining_next;
            i += 1;
            if i == settings.iterations {
                break;
            }
        }
    }
    latency_sample.payload_mut().used_size = settings.iterations;
    latency_sample.send()?;

    println!("Leader finished!");

    // FIXME the samples are not received when the process is gone
    std::thread::sleep(Duration::from_secs(2));

    Ok(())
}

// Copyright (c) 2023 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

extern crate alloc;
use alloc::boxed::Box;

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_bb_posix::clock::Time;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let ping_event_service = node
        .service_builder(&"ping".try_into()?)
        .event()
        .open_or_create()?;
    let notifier = ping_event_service.notifier_builder().create()?;

    let pong_event_service = node
        .service_builder(&"pong".try_into()?)
        .event()
        .open_or_create()?;
    let listener = pong_event_service.listener_builder().create()?;

    let ping_pub_sub_service = node
        .service_builder(&"ping".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let publisher = ping_pub_sub_service.publisher_builder().create()?;

    let pong_pub_sub_service = node
        .service_builder(&"pong".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let subscriber = pong_pub_sub_service.subscriber_builder().create()?;

    let mut counter: u64 = 0;
    while node.wait(Duration::from_millis(1000)).is_ok() {
        counter += 1;
        coutln!("Send ping: {counter}");
        let start = Time::now().unwrap();
        publisher.send_copy(counter)?;
        notifier.notify()?;
        listener.timed_wait(
            |_| {
                while let Ok(Some(sample)) = subscriber.receive() {
                    let elapsed = start.elapsed().unwrap();
                    coutln!("Received pong: {} after {:?}", *sample, elapsed);
                }
            },
            Duration::from_millis(500),
        )?;
    }

    coutln!("exit");

    Ok(())
}

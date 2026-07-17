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

mod rpc;

use clap::Parser;

extern crate alloc;
use alloc::boxed::Box;

use core::net::Ipv4Addr;
use core::time::Duration;

use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

type Connection = etm::client::Connection<rpc::Request, rpc::Response, rpc::Error>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server IP address
    #[arg(long)]
    #[arg(long, default_value = "127.0.0.1")]
    server: Ipv4Addr,
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let args = Args::parse();

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let pong_event_service = node
        .service_builder(&"pong".try_into()?)
        .event()
        .open_or_create()?;
    let notifier = pong_event_service.notifier_builder().create()?;

    let ping_event_service = node
        .service_builder(&"ping".try_into()?)
        .event()
        .open_or_create()?;
    let listener = ping_event_service.listener_builder().create()?;

    let pong_pub_sub_service = node
        .service_builder(&"pong".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let publisher = pong_pub_sub_service.publisher_builder().create()?;

    let ping_pub_sub_service = node
        .service_builder(&"ping".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let subscriber = ping_pub_sub_service.subscriber_builder().create()?;

    let mut etm_client = Connection::new(args.server, rpc::SERVICE_CONNECTION_REQUEST_PORT, None)
        .expect("Could not open connection to server");

    let service = etm::Service::entity(
        rpc::SERVICE_ID.to_string(),
        rpc::ProtocolVersion::entity().version(),
    );

    if !etm_client.compatibility_check(service) {
        coutln!("Compatibility Check with server failed!");
        panic!();
    }

    while node.wait(Duration::ZERO).is_ok() {
        listener.timed_wait(
            |_| {
                while let Ok(Some(sample)) = subscriber.receive() {
                    match etm_client.transceive(rpc::Request::Ping(*sample)) {
                        Some(rpc::Response::Pong(n)) => {
                            let _ = publisher.send_copy(n);
                            let _ = notifier.notify();
                        }
                        _ => coutln!("No response from server"),
                    }
                }
            },
            CYCLE_TIME,
        )?;
    }

    coutln!("exit");

    Ok(())
}

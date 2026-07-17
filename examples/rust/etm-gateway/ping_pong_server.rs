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
use rpc::{Request, Response};

use etm::server::{MessageProcessing, Server};

extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;

use core::time::Duration;

use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::port_factory::event;
use iceoryx2::service::port_factory::publish_subscribe;

type IoxService = ipc_threadsafe::Service;

struct PingPongServer {
    _node: Node<IoxService>,
    _pong_event_service: event::PortFactory<IoxService>,
    _ping_event_service: event::PortFactory<IoxService>,
    _pong_pub_sub_service: publish_subscribe::PortFactory<IoxService, u64, ()>,
    _ping_pub_sub_service: publish_subscribe::PortFactory<IoxService, u64, ()>,
    notifier: Notifier<IoxService>,
    listener: Listener<IoxService>,
    publisher: Publisher<IoxService, u64, ()>,
    subscriber: Subscriber<IoxService, u64, ()>,
}

impl PingPongServer {
    fn new() -> Self {
        let node = NodeBuilder::new().create::<IoxService>().unwrap();

        let ping_event_service = node
            .service_builder(&"server-ping".try_into().unwrap())
            .event()
            .open_or_create()
            .unwrap();
        let notifier = ping_event_service.notifier_builder().create().unwrap();

        let pong_event_service = node
            .service_builder(&"server-pong".try_into().unwrap())
            .event()
            .open_or_create()
            .unwrap();
        let listener = pong_event_service.listener_builder().create().unwrap();

        let ping_pub_sub_service = node
            .service_builder(&"server-ping".try_into().unwrap())
            .publish_subscribe::<u64>()
            .open_or_create()
            .unwrap();
        let publisher = ping_pub_sub_service.publisher_builder().create().unwrap();

        let pong_pub_sub_service = node
            .service_builder(&"server-pong".try_into().unwrap())
            .publish_subscribe::<u64>()
            .open_or_create()
            .unwrap();
        let subscriber = pong_pub_sub_service.subscriber_builder().create().unwrap();

        Self {
            _node: node,
            _pong_event_service: pong_event_service,
            _ping_event_service: ping_event_service,
            _pong_pub_sub_service: pong_pub_sub_service,
            _ping_pub_sub_service: ping_pub_sub_service,
            notifier,
            listener,
            publisher,
            subscriber,
        }
    }
}

impl MessageProcessing for PingPongServer {
    type Rq = rpc::Request;
    type Rsp = rpc::Response;
    type E = rpc::Error;

    fn new() -> Arc<Self> {
        Arc::new(Self::new())
    }

    fn execute(&self, _connection_id: u32, rpc: Self::Rq) -> Result<Self::Rsp, Self::E> {
        match rpc {
            Request::Ping(n) => {
                self.publisher.send_copy(n).unwrap();
                self.notifier.notify().unwrap();
                let mut pong = 0;
                let listener_result = self.listener.timed_wait(
                    |_| {
                        while let Ok(Some(sample)) = self.subscriber.receive() {
                            pong = *sample;
                        }
                    },
                    Duration::from_millis(500),
                );

                match listener_result {
                    Ok(n) if n > 0 => (),
                    Ok(_) => {
                        coutln!("No pong received");
                    }
                    Err(e) => {
                        coutln!("Error waiting on listener: {:?}", e);
                    }
                }

                Ok(Response::Pong(pong))
            }
        }
    }
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let etm_server = Server::<PingPongServer>::new(
        rpc::SERVICE_CONNECTION_REQUEST_PORT,
        etm::Service::entity(
            rpc::SERVICE_ID.to_string(),
            rpc::ProtocolVersion::entity().version(),
        ),
    );

    etm_server.run()?;

    coutln!("exit");

    Ok(())
}

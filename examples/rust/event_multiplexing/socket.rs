// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::unix_datagram_socket::*;
use iceoryx2_bb_system_types::file_path::FilePath;

use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let socket_name = FilePath::new(b"mySocket").unwrap();

    let sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    while node.wait(CYCLE_TIME).is_ok() {
        // send some data
        let data: Vec<u8> = vec![1u8, 2u8, 3u8, 4u8, 5u8];
        sender.try_send(data.as_slice()).unwrap();

        println!("[sending data: \"{:?}\"]", data);
    }

    println!("exit");

    Ok(())
}

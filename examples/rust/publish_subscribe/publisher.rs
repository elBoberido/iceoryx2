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

// use core::time::Duration;
//
// extern crate alloc;
// use alloc::boxed::Box;
//
// use examples_common::TransmissionData;
// use iceoryx2::prelude::*;
//
// const CYCLE_TIME: Duration = Duration::from_secs(1);
//
// fn main() -> Result<(), Box<dyn core::error::Error>> {
//     set_log_level_from_env_or(LogLevel::Info);
//
//     let node = NodeBuilder::new().create::<ipc::Service>()?;
//
//     let service = node
//         .service_builder(&"My/Funk/ServiceName".try_into()?)
//         .publish_subscribe::<TransmissionData>()
//         .open_or_create()?;
//     service.service_hash();
//
//     let publisher = service.publisher_builder().create()?;
//
//     let mut counter: u64 = 0;
//
//     while node.wait(CYCLE_TIME).is_ok() {
//         counter += 1;
//         let sample = publisher.loan_uninit()?;
//
//         let sample = sample.write_payload(TransmissionData {
//             x: counter as i32,
//             y: counter as i32 * 3,
//             funky: counter as f64 * 812.12,
//         });
//
//         sample.send()?;
//
//         coutln!("Send sample {counter} ...");
//     }
//
//     coutln!("exit");
//
//     Ok(())
// }
//

use libc::{
    c_int, c_void, size_t, sockaddr_un, socklen_t, timeval, AF_UNIX, SOCK_DGRAM, SOL_SOCKET,
    SO_RCVBUF, SO_RCVTIMEO, SO_SNDBUF, SO_SNDTIMEO,
};
use std::ffi::CString;
use std::io::{Error, Result};
use std::mem::{size_of, zeroed};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Create socket
    let sock_fd = unsafe { libc::socket(AF_UNIX, SOCK_DGRAM, 0) };
    if sock_fd < 0 {
        return Err(Error::last_os_error());
    }

    let mut sndbuf: c_int = 0;
    let mut sndbuf_len = size_of::<c_int>() as socklen_t;
    let mut rcvbuf: c_int = 0;
    let mut rcvbuf_len = size_of::<c_int>() as socklen_t;
    unsafe {
        libc::getsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_SNDBUF,
            &mut sndbuf as *mut _ as *mut c_void,
            &mut sndbuf_len,
        );
        libc::getsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_RCVBUF,
            &mut rcvbuf as *mut _ as *mut c_void,
            &mut rcvbuf_len,
        );
    }
    println!("#### [OLD] SNDBUF: {}, RCVBUF: {}", sndbuf, rcvbuf);

    // Set socket options: SNDBUF and RCVBUF
    let sndbuf_size: c_int = 1024 * 16;
    let rcvbuf_size: c_int = 1024 * 4;
    unsafe {
        libc::setsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_SNDBUF,
            &sndbuf_size as *const _ as *const c_void,
            size_of::<c_int>() as socklen_t,
        );
        libc::setsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_RCVBUF,
            &rcvbuf_size as *const _ as *const c_void,
            size_of::<c_int>() as socklen_t,
        );
    }

    // Set timeouts (1 second)
    let timeout = timeval {
        tv_sec: 1,
        tv_usec: 0,
    };
    unsafe {
        libc::setsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_RCVTIMEO,
            &timeout as *const _ as *const c_void,
            size_of::<timeval>() as socklen_t,
        );
        libc::setsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_SNDTIMEO,
            &timeout as *const _ as *const c_void,
            size_of::<timeval>() as socklen_t,
        );
    }

    // Get current SNDBUF and RCVBUF
    let mut sndbuf: c_int = 0;
    let mut sndbuf_len = size_of::<c_int>() as socklen_t;
    let mut rcvbuf: c_int = 0;
    let mut rcvbuf_len = size_of::<c_int>() as socklen_t;
    unsafe {
        libc::getsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_SNDBUF,
            &mut sndbuf as *mut _ as *mut c_void,
            &mut sndbuf_len,
        );
        libc::getsockopt(
            sock_fd,
            SOL_SOCKET,
            SO_RCVBUF,
            &mut rcvbuf as *mut _ as *mut c_void,
            &mut rcvbuf_len,
        );
    }
    println!("#### [NEW] SNDBUF: {}, RCVBUF: {}", sndbuf, rcvbuf);

    // Bind socket to a path
    let sock_path = c"/tmp/ud_sock_3";
    let sock_path = unsafe {
        let bytes = sock_path.to_bytes_with_nul();
        std::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len())
    };
    let addr = sockaddr_un {
        sun_family: AF_UNIX as _,
        sun_path: [0i8; 108],
        #[cfg(target_os = "freebsd")]
        sun_path: [0i8; 180],
        #[cfg(target_os = "freebsd")]
        sun_length: 0,
    };
    let mut addr = addr;
    addr.sun_path[..sock_path.len()].copy_from_slice(&sock_path);
    let addr_len = size_of::<sockaddr_un>() as socklen_t;
    #[cfg(target_os = "freebsd")]
    addr.sun_length = addr_len;

    if unsafe { libc::bind(sock_fd, &addr as *const _ as *const _, addr_len) } < 0 {
        return Err(Error::last_os_error());
    }

    // Spawn sender thread
    let sender_fd = sock_fd;
    thread::spawn(move || {
        let msg = CString::new("Hello from sender!").unwrap();
        loop {
            let bytes_sent = unsafe {
                libc::sendto(
                    sender_fd,
                    msg.as_ptr() as *const c_void,
                    msg.as_bytes().len(),
                    0,
                    &addr as *const _ as *const _,
                    addr_len,
                )
            };
            if bytes_sent < 0 {
                eprintln!("Send error: {:?}", Error::last_os_error());
                break;
            }
            println!("Sent {} bytes", bytes_sent);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Receiver loop
    let mut buf = [0u8; 1024];
    loop {
        let bytes_received = unsafe {
            libc::recvfrom(
                sock_fd,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if bytes_received < 0 {
            eprintln!("Receive error: {:?}", Error::last_os_error());
            break;
        }
        println!(
            "Received {} bytes: {:?}",
            bytes_received,
            &buf[..bytes_received as usize]
        );
        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

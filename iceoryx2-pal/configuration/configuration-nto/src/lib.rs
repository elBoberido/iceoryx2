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

#![no_std]

pub const GLOBAL_CONFIG_PATH: &[u8] = b"/etc";
pub const USER_CONFIG_PATH: &[u8] = b".config";
pub const TEMP_DIRECTORY: &[u8] = b"/data/iceoryx2/tmp/";
pub const TEST_DIRECTORY: &[u8] = b"/data/iceoryx2/tests/";
pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"/dev/shmem/";
pub const PATH_SEPARATOR: u8 = b'/';
pub const ROOT: &[u8] = b"/";
pub const REQUIRED_SOCKET_DIRECTORY: Option<&[u8]> = None;
pub const ICEORYX2_ROOT_PATH: &[u8] = b"/data/iceoryx2/";
pub const FILENAME_LENGTH: usize = 255;
pub const PATH_LENGTH: usize = 255;
pub const AT_LEAST_TIMING_VARIANCE: f32 = 0.25;

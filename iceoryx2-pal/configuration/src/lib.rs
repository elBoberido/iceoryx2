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

#[cfg(all(
    not(iceoryx2_custom_pal_config),
    not(iceoryx2_custom_cal_recommended),
    not(feature = "custom"),
    not(target_os = "windows"),
    not(target_os = "nto")
))]
pub use iceoryx2_pal_configuration_generic::*;

#[cfg(all(
    not(iceoryx2_custom_pal_config),
    not(iceoryx2_custom_cal_recommended),
    not(feature = "custom"),
    target_os = "nto"
))]
pub use iceoryx2_pal_configuration_nto::*;

#[cfg(all(
    not(iceoryx2_custom_pal_config),
    not(iceoryx2_custom_cal_recommended),
    not(feature = "custom"),
    target_os = "windows"
))]
pub use iceoryx2_pal_configuration_windows::*;

// #[cfg(feature = "custom")]
// pub use iceoryx2_pal_configuration_custom::*;

// #[cfg(feature = "custom")]
// pub use iceoryx::*;

#[cfg(iceoryx2_custom_pal_config)]
pub use iceoryx::*;

#[cfg(iceoryx2_custom_cal_recommended)]
pub use iceoryx::*;

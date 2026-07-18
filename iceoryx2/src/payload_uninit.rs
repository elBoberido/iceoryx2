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

use core::mem::MaybeUninit;

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

/// This is a small wrapper around a [`MaybeUninit`] mainly for internal use
#[repr(transparent)]
#[derive(Debug)]
pub struct PayloadUninit<T> {
    pub(crate) inner: MaybeUninit<T>,
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for PayloadUninit<T> {}

impl<T: Clone + Copy> Clone for PayloadUninit<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy> Copy for PayloadUninit<T> {}

impl<T> PayloadUninit<T> {
    /// Creates a new [`PayloadUninit`]
    pub const fn new(value: MaybeUninit<T>) -> Self {
        Self { inner: value }
    }

    /// Gets a pointer to the contained value.
    ///
    /// Attention: Reading from this pointer or turning it into a reference is
    /// undefined behavior unless the inner [`MaybeUninit`] is initialized.
    pub const fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    /// Gets a mutable pointer to the contained value.
    ///
    /// Attention: Reading from this pointer or turning it into a reference is
    /// undefined behavior unless the inner [`MaybeUninit`] is initialized.
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr()
    }

    /// Sets the value of the inner [`MaybeUninit`]
    pub const fn write(&mut self, val: T) -> &mut T {
        self.inner.write(val)
    }
}

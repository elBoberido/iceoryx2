// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container::vector::relocatable_vec::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_testing::assert_that;

#[test]
#[should_panic]
fn double_init_call_causes_panic() {
    const CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<u128>::const_memory_size(CAPACITY);
    let mut memory = [0u8; MEM_SIZE];
    let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
    let mut sut = unsafe { RelocatableVec::<u128>::new_uninit(CAPACITY) };
    unsafe { sut.init(&bump_allocator).expect("sut init failed") };

    unsafe { sut.init(&bump_allocator).expect("sut init failed") };
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn panic_is_called_in_debug_mode_if_vec_is_not_initialized() {
    const CAPACITY: usize = 12;
    let mut sut = unsafe { RelocatableVec::<u8>::new_uninit(CAPACITY) };
    assert_that!(sut.push(0), is_ok);
}

#[test]
fn two_vectors_with_same_content_are_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    assert_that!(sut_1, eq sut_2);
}

#[test]
fn two_vectors_with_different_content_are_not_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    sut_2[5] = 0;

    assert_that!(sut_1, ne sut_2);
}

#[test]
fn two_vectors_with_different_len_are_not_equal() {
    const SUT_CAPACITY: usize = 12;
    const MEM_SIZE: usize = RelocatableVec::<usize>::const_memory_size(SUT_CAPACITY);
    let mut memory_1 = [0u8; MEM_SIZE];
    let mut memory_2 = [0u8; MEM_SIZE];
    let bump_allocator_1 = BumpAllocator::new(memory_1.as_mut_ptr());
    let bump_allocator_2 = BumpAllocator::new(memory_2.as_mut_ptr());
    let mut sut_1 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_1.init(&bump_allocator_1).unwrap() };
    let mut sut_2 = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
    unsafe { sut_2.init(&bump_allocator_2).unwrap() };

    for n in 0..SUT_CAPACITY {
        assert_that!(sut_1.push(4 * n + 3), is_ok);
        assert_that!(sut_2.insert(n, 4 * n + 3), is_ok);
    }

    sut_2.pop();

    assert_that!(sut_1, ne sut_2);
}

#[test]
fn ptr_provenance_test_stack_separated() {
    const CAPACITY: usize = 10;
    let mut memory = [0u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)];
    let mut sut = unsafe { RelocatableVec::<usize>::new_uninit(CAPACITY) };

    let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
    // no warning/error when using heap
    // let mut sut = Box::new(unsafe { RelocatableVec::<usize>::new_uninit(CAPACITY) });
    unsafe { sut.init(&bump_allocator).unwrap() };

    assert_that!(sut.push(0), is_ok);
    // drop(sut);

    unsafe {
        core::ptr::drop_in_place(&mut sut as *mut _);
    }
    core::mem::forget(sut);
}

#[test]
fn ptr_provenance_test_stack_stack_bundled() {
    const CAPACITY: usize = 10;

    struct Bundle {
        sut: RelocatableVec<usize>,
        memory: [u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)],
    }

    let mut b = Bundle {
        sut: unsafe { RelocatableVec::<usize>::new_uninit(CAPACITY) },
        memory: [0u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)],
    };

    let bump_allocator = BumpAllocator::new(b.memory.as_mut_ptr());

    unsafe { b.sut.init(&bump_allocator).unwrap() };

    assert_that!(b.sut.push(0), is_ok);
    // drop(b.sut);

    unsafe {
        core::ptr::drop_in_place(&mut b.sut as *mut _);
    }
    core::mem::forget(b.sut);
}

#[test]
fn ptr_provenance_test_heap_bundled() {
    const CAPACITY: usize = 10;

    #[repr(C)]
    struct Bundle {
        sut: RelocatableVec<usize>,
        memory: [u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)],
    }

    let mut b = Box::new(Bundle {
        sut: unsafe { RelocatableVec::<usize>::new_uninit(CAPACITY) },
        memory: [0u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)],
    });

    let bump_allocator = BumpAllocator::new(b.memory.as_mut_ptr());

    unsafe { b.sut.init(&bump_allocator).unwrap() };

    assert_that!(b.sut.push(0), is_ok);
    // drop(b.sut);

    unsafe {
        core::ptr::drop_in_place(&mut b.sut as *mut _);
    }
    core::mem::forget(b.sut);
}

#[test]
fn ptr_provenance_test_heap_separate() {
    const CAPACITY: usize = 10;

    let mut shm = [0u8; RelocatableVec::<usize>::const_memory_size(CAPACITY) * 10];

    let shm_alloc = BumpAllocator::new(shm.as_mut_ptr());

    let mut memory = shm_alloc
        .allocate(core::alloc::Layout::new::<
            [u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)],
        >())
        .unwrap()
        .cast::<[u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)]>();

    unsafe { memory.write([0u8; RelocatableVec::<usize>::const_memory_size(CAPACITY)]) };

    let mut sut = shm_alloc
        .allocate(core::alloc::Layout::new::<RelocatableVec<usize>>())
        .unwrap()
        .cast::<RelocatableVec<usize>>();

    unsafe { sut.write(RelocatableVec::<usize>::new_uninit(CAPACITY)) };

    unsafe {
        let bump_allocator = BumpAllocator::new((*memory.as_mut()).as_mut_ptr());

        sut.as_mut().init(&bump_allocator).unwrap();

        assert_that!((*sut.as_ptr()).push(0), is_ok);
        core::ptr::drop_in_place(sut.as_ptr());
    }
}

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use blog_os::memory::{self, BootInfoFrameAllocator};
use blog_os::{allocator, hlt_loop, test};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    blog_os::init();
    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed");

    test_main();
    hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

test!(simple_alloc {
    let heap_val = Box::new(41);
    assert_eq!(*heap_val, 41);
});

test!(large_vec {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u32>(), (n-1) * n / 2);
});

test!(many_boxes {
    for i in 0..10000 {
        let val = Box::new(i);
        assert_eq!(*val, i);
    }
});

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
pub mod serial;
#[macro_use]
pub mod testing;
#[macro_use]
pub mod vga_buffer;
pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;

use bootloader::BootInfo;
use x86_64::structures::paging::{FrameAllocator, MapperAllSizes, Size4KiB};

use linked_list_allocator::LockedHeap;
pub use testing::*;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init(boot_info: &'static BootInfo) -> (impl MapperAllSizes, impl FrameAllocator<Size4KiB>) {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pics();

    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::new(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    (mapper, frame_allocator)
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc error: {:?}", layout);
}

// For integration testing

#[cfg(test)]
use bootloader::entry_point;
#[cfg(test)]
use core::panic::PanicInfo;

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[cfg(test)]
entry_point!(kernel_main);

#[cfg(test)]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop();
}

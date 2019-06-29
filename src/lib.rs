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

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
#[cfg(test)]
use core::panic::PanicInfo;

use linked_list_allocator::LockedHeap;
pub use testing::*;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_pics();
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Alloc error: {:?}", layout);
}

// For integration testing
#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

#[cfg(test)]
entry_point!(kernel_main);

#[cfg(test)]
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

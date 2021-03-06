#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]
#![feature(alloc_layout_extra)]

extern crate alloc;

#[macro_use]
pub mod serial;
#[macro_use]
pub mod testing;
#[macro_use]
pub mod vga_buffer;
pub mod allocator;
pub mod event;
pub mod gdt;
pub mod interrupts;
pub mod memory;

use bootloader::BootInfo;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    structures::paging::{FrameAllocator, MapperAllSizes, Size4KiB},
    VirtAddr,
};

pub use testing::*;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// Initialization procedure should only ever run once, so we use a flag to ensure that
lazy_static! {
    static ref INIT_FLAG: Mutex<bool> = Mutex::new(false);
}

pub fn init(
    boot_info: &'static BootInfo,
) -> Result<(impl MapperAllSizes, impl FrameAllocator<Size4KiB>), ()> {
    if *INIT_FLAG.lock() {
        Err(())
    } else {
        *INIT_FLAG.lock() = true;

        gdt::init();
        interrupts::init_idt();
        interrupts::init_pics();

        let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
        let mut mapper = unsafe { memory::init(physical_memory_offset) };
        let mut frame_allocator =
            unsafe { memory::BootInfoFrameAllocator::new(&boot_info.memory_map) };
        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");

        Ok((mapper, frame_allocator))
    }
}

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
    init(boot_info).unwrap();
    test_main();
    hlt_loop();
}

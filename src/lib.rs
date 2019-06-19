#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

#[macro_use]
pub mod serial;
#[macro_use]
pub mod testing;
#[macro_use]
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

#[cfg(test)]
use core::panic::PanicInfo;
pub use testing::*;

// For integration testing

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    testing::test_panic_handler(info)
}

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

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

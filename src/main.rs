#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use]
mod serial;
#[macro_use]
mod testing;
#[macro_use]
mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Failed");
    serial_println!("Error: {}", info);

    testing::exit_qemu(testing::QemuExitCode::Failed);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    //for i in 0..30 {
    //println!("{}", i);
    //}
    //println!();
    //panic!("Message");

    #[cfg(test)]
    test_main();

    loop {}
}

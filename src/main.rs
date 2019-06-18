#![no_main]
#![no_std]

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    for i in 0..30 {
        println!("{}", i);
    }
    println!();
    panic!("Message");

    loop {}
}

#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[cfg(not(test))]
use blog_os::println;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    blog_os::hlt_loop();
}

#[panic_handler]
#[cfg(test)]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    #[cfg(test)]
    {
        blog_os::init(boot_info).unwrap();
        test_main();
    }

    #[cfg(not(test))]
    {
        use alloc::{boxed::Box, vec};

        blog_os::init(boot_info).unwrap();

        println!("Hello World!");
        let x = Box::new(41);
        println!("Heap box at {:p}", x);
        let y = vec![1, 2, 3];
        println!("vec {:?}", y);
    }

    blog_os::hlt_loop();
}

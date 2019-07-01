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
        blog_os::hlt_loop();
    }

    #[cfg(not(test))]
    {
        blog_os::init(boot_info).unwrap();

        use alloc::boxed::Box;

        //use blog_os::event::keyboard;
        //let printer = keyboard::KeyPrinter {};
        //let mut runner = keyboard::KEYBOARD_EVENT_DISPATCHER.lock();

        //runner.add_listener(Box::new(printer));
        //loop {
        //runner.poll();
        //// Need this instruction to prevent tight polling from starving the interrupts
        //x86_64::instructions::hlt();
        //}

        use blog_os::event::timer;
        let printer = timer::TimerPrinter {};
        let mut runner = timer::TIMER_EVENT_DISPATCHER.lock();

        runner.add_listener(Box::new(printer));
        loop {
            runner.poll();
            // Need this instruction to prevent tight polling from starving the interrupts
            x86_64::instructions::hlt();
        }
    }
}

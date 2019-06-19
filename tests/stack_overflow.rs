#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use blog_os::{exit_qemu, gdt, serial_println, test, QemuExitCode};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    blog_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _err: u64,
) {
    serial_println!("[Ok]");
    exit_qemu(QemuExitCode::Success);
    panic!("WTF should have exited");
}

#[allow(unreachable_code)]
mod tests {
    use super::*;

    test!(handle_stack_overflow {
        gdt::init();
        init_test_idt();

        stack_overflow();

        panic!("Execution continued after stack overflow");
    });

    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }
}

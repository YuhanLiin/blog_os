#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(panic_info_message)]

use blog_os::{exit_qemu, serial_println, test, QemuExitCode};
use core::fmt;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

const MSG: &str = "Test Message";

lazy_static! {
    static ref LINE: Mutex<Option<u32>> = Mutex::new(None);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

fn fail(error: &str) -> ! {
    serial_println!("[failed]");
    serial_println!("{}", error);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn check_location(info: &PanicInfo) {
    let location = info.location().unwrap_or_else(|| fail("no location"));

    if location.file() != file!() {
        fail("file name wrong");
    }
    if location.line()
        != LINE
            .lock()
            .unwrap_or_else(|| fail("no expected line number"))
    {
        fail("line number wrong");
    }
}

struct CompareMessage {
    expected: &'static str,
}

impl fmt::Write for CompareMessage {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.expected.starts_with(s) {
            self.expected = &self.expected[s.len()..];
        } else {
            fail("message not equal to expected message");
        }
        Ok(())
    }
}

fn check_message(info: &PanicInfo) {
    use core::fmt::Write;

    let msg = info.message().unwrap_or_else(|| fail("no message"));
    let mut cmp = CompareMessage { expected: MSG };

    write!(&mut cmp, "{}", msg).unwrap_or_else(|_| fail("write failed"));
    if !cmp.expected.is_empty() {
        fail("message shorter than expected");
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    check_message(info);
    check_location(info);
    serial_println!("[Ok]");

    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[allow(unreachable_code)]
mod test {
    use super::*;
    test!(handle_panic {
        // Has to be on the same line
        *LINE.lock() = Some(line!()); panic!("{}", MSG);
    });
}

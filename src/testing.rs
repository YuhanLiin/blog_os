use core::panic::PanicInfo;

#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("Failed");
    serial_println!("Error: {}", info);

    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[macro_export]
macro_rules! test {
    ($name:ident { $($body:tt)* }) => {
        #[test_case]
        fn $name() {
            $crate::serial_print!("{}::{}... ", module_path!(), stringify!($name));
            {$($body)*}
            $crate::serial_println!("[Ok]");
        }
    }
}

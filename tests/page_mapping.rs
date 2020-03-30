#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use blog_os::test;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    structures::paging::{page::Size4KiB, MapperAllSizes, Page},
    PhysAddr, VirtAddr,
};

lazy_static! {
    static ref BOOT_INFO: Mutex<Option<&'static BootInfo>> = Mutex::new(None);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    *BOOT_INFO.lock() = Some(boot_info);
    test_main();
    blog_os::hlt_loop();
}

test!(page_map {
    let boot_info = &*BOOT_INFO.lock().unwrap();
    let (mut mapper, mut frame_allocator) = blog_os::init(boot_info).unwrap();

    let addresses = [
        // some code page
        0x20010a,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        assert!(phys.is_some());
    }

    // Identity mapped VGA buffer page
    assert_eq!(mapper.translate_addr(VirtAddr::new(0xb8000)), Some(PhysAddr::new(0xb8000)));

    // Should map to physical address of 0x0
    let offset = boot_info.physical_memory_offset;
    assert_eq!(mapper.translate_addr(VirtAddr::new(offset)), Some(PhysAddr::new(0x0)));

    // Create a new page mapping and assert that it's actually mapped to that page
    let new_page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    let old_page = Page::<Size4KiB>::containing_address(VirtAddr::new(0xb8000));
    blog_os::memory::create_mapping(
        PhysAddr::new(0xb8000),
        new_page,
        &mut mapper,
        &mut frame_allocator,
    );

    let new_page_ptr: *mut u64 = new_page.start_address().as_mut_ptr();
    let old_page_ptr: *const u64 = old_page.start_address().as_ptr();
    unsafe { new_page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
    unsafe { assert_eq!(old_page_ptr.offset(400).read_volatile(), 0x_f021_f077_f065_f04e); }
});

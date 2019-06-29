#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

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
    blog_os::init();

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        use alloc::{boxed::Box, vec};
        use blog_os::allocator;
        use blog_os::memory;

        println!("Hello World!");

        let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
        let mut frame_allocator =
            unsafe { memory::BootInfoFrameAllocator::new(&boot_info.memory_map) };

        allocator::init_heap(&mut mapper, &mut frame_allocator)
            .expect("heap initialization failed");
        let x = Box::new(41);
        println!("Heap box at {:p}", x);
        let y = vec![1, 2, 3];
        println!("vec {:?}", y);

        let addresses = [
            // the identity-mapped vga buffer page
            0xb8000,
            // some code page
            0x20010a,
            // some stack page
            0x57ac_001f_fe48,
            // virtual address mapped to physical address 0
            boot_info.physical_memory_offset,
        ];

        use x86_64::{
            structures::paging::{MapperAllSizes, Page},
            PhysAddr, VirtAddr,
        };

        for &address in &addresses {
            let virt = VirtAddr::new(address);
            let phys = mapper.translate_addr(virt);
            println!("{:?} -> {:?}", virt, phys);
        }

        let page = Page::containing_address(VirtAddr::new(0xdeadbeef));

        memory::create_mapping(
            PhysAddr::new(0xb8000),
            page,
            &mut mapper,
            &mut frame_allocator,
        );

        let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
        unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
    }

    blog_os::hlt_loop();
}

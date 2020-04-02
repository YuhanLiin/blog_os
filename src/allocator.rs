pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: Locked<fixed_size_block::FixedSizeBlockAllocator> =
    Locked::new(fixed_size_block::FixedSizeBlockAllocator::new());

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// Aligns the address upwards to a specific alignment boundary
///
/// `align` should be power of 2
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec::Vec};

    test!(simple_alloc {
        let heap_val = Box::new(41);
        assert_eq!(*heap_val, 41);
    });

    test!(large_vec {
        let n = 1000;
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(i);
        }
        assert_eq!(vec.iter().sum::<u32>(), (n-1) * n / 2);
    });

    test!(many_boxes {
        for i in 0..10000 {
            let val = Box::new(i);
            assert_eq!(*val, i);
        }
    });

    test!(many_boxes_long_lived {
        let long_lived = Box::new(10);
        for i in 0..10000 {
            let val = Box::new(i);
            assert_eq!(*val, i);
        }
        assert_eq!(*long_lived, 10);
    });
}

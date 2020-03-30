use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    UnusedPhysFrame,
};
use x86_64::{registers::control::Cr3, PhysAddr, VirtAddr};

// Physical memory must be mapped at offset.
// Must only be called once to avoid aliasing &mut PageTable.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();

    let page_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_ptr // unsafe
}

// Complete physical memory must be mapped at the offset.
// Must only be called once to avoid aliasing &mut PageTables.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // Usable regions in the memory map should actually be unused
    pub unsafe fn new(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_map
            .iter()
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        let frame = unsafe {
            self.usable_frames()
                .nth(self.next)
                .map(|f| UnusedPhysFrame::new(f))
        };
        self.next += 1;
        frame
    }
}

pub fn create_mapping(
    addr: PhysAddr,
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(addr);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let result =
        unsafe { mapper.map_to(page, UnusedPhysFrame::new(frame), flags, frame_allocator) };
    result.expect("map_to failed").flush();
}

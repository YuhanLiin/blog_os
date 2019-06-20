use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{
    FrameAllocator, MappedPageTable, Mapper, MapperAllSizes, Page, PageTable, PageTableFlags,
    PhysFrame, Size4KiB,
};
use x86_64::{registers::control::Cr3, PhysAddr, VirtAddr};

// Physical memory must be mapped at offset.
// Must only be called once to avoid aliasing &mut PageTable.
unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);

    let page_ptr = virt.as_mut_ptr();
    &mut *page_ptr // unsafe
}

// Complete physical memory must be mapped at the offset.
// Must only be called once to avoid aliasing &mut PageTables.
pub unsafe fn init(physical_memory_offset: u64) -> impl MapperAllSizes {
    let level_4_table = active_level_4_table(physical_memory_offset);
    let phys_to_virt = move |frame: PhysFrame| -> *mut PageTable {
        let phys = frame.start_address().as_u64();
        let virt = VirtAddr::new(phys + physical_memory_offset);
        virt.as_mut_ptr()
    };

    MappedPageTable::new(level_4_table, phys_to_virt)
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
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
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

    let result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };
    result.expect("map_to failed").flush();
}

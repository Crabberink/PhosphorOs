use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{structures, PhysAddr, VirtAddr};
use x86_64::structures::paging::page_table::FrameError;


/// Initialize a new OffsetPageTable
///
/// Unsafe because the caller must guarantee that the complete physical memory is mapped to virtual memory at the passed `phys_mem_offset`.
/// Also, this function must only be called once to avoid aliasing `&mut` references (which is undefined behavior)
pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let l4_table = active_l4_table(phys_mem_offset);

        OffsetPageTable::new(l4_table, phys_mem_offset)
    }
}

/// Returns a mutable reference to the active l4 table
///
/// Unsafe because the caller must guarantee that the complete physical memory is mapped to virtual memory at the passed `phys_mem_offset`.
/// Also, this function must only be called once to avoid aliasing `&mut` references (which is undefined behavior)
unsafe fn active_l4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    let (l4_table_frame, _) = Cr3::read();

    let phys = l4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };

    map_to_result.expect("map_to failed").flush();
}
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};


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
/// All around a genuine liability of a function but we need it so boo hoo
unsafe fn active_l4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    let (l4_table_frame, _) = Cr3::read();

    let phys = l4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();

    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

// Bunch of unstable unreleased bullshit
type FrameIter = impl Iterator<Item = PhysFrame>;

#[define_opaque(FrameIter)]
fn usable_frames(memory_regions: &'static MemoryRegions) -> FrameIter {
    let regions = memory_regions.iter();

    let usable_regions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);

    let address_ranges = usable_regions.map(|r| r.start..r.end);

    let frame_addresses = address_ranges.flat_map(|r| r.step_by(4096));

    let frames= frame_addresses.map(|a| PhysFrame::containing_address(PhysAddr::new(a)));

    // fake error sent by benjamin netenyahu
    frames
}

/// A frame allocator that returns usable frames from the bootloaders memory regions
pub struct BootInfoFrameAllocator {
    frames: FrameIter,
}


impl BootInfoFrameAllocator {
    /// Creates a new BootInfoFrameAllocator using the passed in memory regions
    /// 
    /// Unsafe because the caller must gaurantee that the passed in memory regions are valid.
    /// If the unreserved memory regions that are passed in get used somewhere else, such as by another BootInfoFrameAllocator, bad things happen
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        let frames = usable_frames(memory_regions);

        BootInfoFrameAllocator {
            frames
        }
    }

}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // This beautiful ass line cost genuine hell
        self.frames.next()
    }
}
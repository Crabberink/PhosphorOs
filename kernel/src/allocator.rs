use alloc::alloc::*;
use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use core::{panic, ptr::*};
use core::prelude::rust_2024::*;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: VirtAddr = VirtAddr::new(0x0000_4444_4444_0000);
pub const HEAP_SIZE: usize = (1024) * 100; // 100 KiB

pub fn init_heap(
    mem_mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>>{
    let page_range = {
        let heap_end = HEAP_START + HEAP_SIZE as u64 - 1;
        
        let heap_start_page: Page<Size4KiB> = Page::containing_address(HEAP_START);
        let heap_end_page: Page<Size4KiB> = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT |PageTableFlags::WRITABLE;

        unsafe {
            mem_mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        let mut alloc = ALLOCATOR.lock();
        alloc.init(HEAP_START.as_mut_ptr(), HEAP_SIZE);
    }
    
    Ok(())
}

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("OH GOD OH FUCK")
    }
}
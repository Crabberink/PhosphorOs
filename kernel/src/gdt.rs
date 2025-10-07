use crate::lazy_static;
use x86_64::{instructions::tables::load_tss, registers::segmentation::{Segment, CS, DS, SS}, structures::{gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector}, tss::TaskStateSegment}, VirtAddr};

pub fn setup_gdt() {
    GDT_SELECTORS.gdt.load();

    unsafe {
        CS::set_reg(GDT_SELECTORS.cs);

        SS::set_reg(GDT_SELECTORS.ds);
        DS::set_reg(GDT_SELECTORS.ds);
        
        load_tss(GDT_SELECTORS.tss);
    }
}

struct GDTSelectors {
    gdt: GlobalDescriptorTable,
    cs: SegmentSelector,
    ds: SegmentSelector,
    tss: SegmentSelector,
}

lazy_static! {
    static ref GDT_SELECTORS: GDTSelectors = {
        let mut gdt = GlobalDescriptorTable::new();
        let cs = gdt.append(Descriptor::kernel_code_segment());
        let ds = gdt.append(Descriptor::kernel_data_segment());

        let tss = gdt.append(Descriptor::tss_segment(&TSS));

        GDTSelectors { gdt: gdt, cs: cs, ds:ds, tss: tss}
    };
}


pub const DOUBLE_FAULT_STACK_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_STACK_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };

        tss
    };
}
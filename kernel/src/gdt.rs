use core::ptr::addr_of;

use lazy_static::lazy_static;
use x86_64::{
    segment_selector::SegmentSelector,
    structures::{Descriptor, GlobalDescriptorTable, TaskStateSegment},
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::default();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { addr_of!(STACK) as usize };

            stack_start + STACK_SIZE
        };

        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::default();

        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {
            code_selector,
            data_selector,
            tss_selector,
        })
    };
}

#[derive(Debug)]
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    log::trace!("initialising gdt");

    GDT.0.load();
    log::trace!("\t* loaded GDT");

    unsafe {
        GDT.1.code_selector.write_cs();
        GDT.1.data_selector.write_ss();
        log::trace!("\t* updated CS and SS");

        GDT.1.tss_selector.load_tss();
        log::trace!("\t* loaded TSS");
    }

    log::trace!("gdt initialised");
}

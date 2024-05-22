use core::{arch::asm, fmt::Debug};

use bitflags::bitflags;
use crabstd::mutex::Mutex;
use lazy_static::lazy_static;
use x86_64::{
    registers::CR2,
    structures::{ExceptionStackFrame, InterruptDescriptorTable},
};

use self::pic::ChainedPics;
use crate::{gdt, print, println};

mod pic;
mod syscall;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::default();

        idt.divide_error.set(divide_by_zero_handler);
        idt.breakpoint.set(breakpoint_handler);
        idt.invalid_opcode.set(invalid_opcode_handler);
        idt.page_fault.set(page_fault_handler);
        idt.general_protection_fault
            .set(general_protection_fault_handler);
        unsafe {
            idt.double_fault
                .set(double_fault)
                .set_ist_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer as u8].set(timer_interrupt_handler);
        idt[0x80].set(syscall_handler);

        idt
    };
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: ExceptionStackFrame) {
    log::error!("EXCEPTION: DIVIDE BY ZERO\n{}", stack_frame);
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{}", stack_frame);

    x86_64::hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: ExceptionStackFrame) {
    log::error!(
        "EXCEPTION: INVALID OPCODE at {:#X}\n{}",
        stack_frame.instruction_pointer,
        stack_frame
    );
    println!(
        "\nEXCEPTION: INVALID OPCODE at {:#X}\n{}",
        stack_frame.instruction_pointer, stack_frame
    );

    x86_64::hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: ExceptionStackFrame) {
    log::info!(
        "EXCEPTION: BREAKPOINT at {:#X}\n{}",
        stack_frame.instruction_pointer,
        stack_frame
    );
    println!(
        "\nEXCEPTION: BREAKPOINT at {:#X}\n{}",
        stack_frame.instruction_pointer, stack_frame
    );
}

extern "x86-interrupt" fn double_fault(stack_frame: ExceptionStackFrame, err: u64) -> ! {
    log::error!("DOUBLE FAULT with err {}\n{}", err, stack_frame);
    panic!("\nDOUBLE FAULT with err {}\n{}", err, stack_frame);
}

bitflags! {
    #[derive(Debug)]
    struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: ExceptionStackFrame, error_code: u64) {
    log::error!(
        "EXCEPTION: PAGE FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        PageFaultErrorCode::from_bits(error_code).unwrap(),
        stack_frame
    );

    x86_64::hlt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: ExceptionStackFrame,
    error_code: u64,
) {
    log::error!(
        "EXCEPTION: GENERAL PROTECTION FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        error_code,
        stack_frame
    );
    println!(
        "\nEXCEPTION: GENERAL PROTECTION FAULT while accessing {:#X}\
        \nerror code: {:?}\n{}",
        CR2::read(),
        error_code,
        stack_frame
    );

    x86_64::hlt_loop();
}

/// Stub interrupt handler that simply jumps to the correct syscall based on the value in `rax`
#[naked]
extern "x86-interrupt" fn syscall_handler(_stack_frame: ExceptionStackFrame) {
    // make sure to mask rax to prevent jumping outside the syscall table
    unsafe {
        asm!(
            "and rax, 0xFF",
            "jmp [{} + rax*8]",
            sym syscall::SYSCALL_TABLE,
            options(noreturn)
        );
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: ExceptionStackFrame) {
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

pub fn init() {
    log::trace!("initialising interrupts");

    IDT.load();
    log::trace!("\t* loaded IDT");

    unsafe {
        PICS.lock().init();
    }
    log::trace!("\t* initialised PIC");

    x86_64::interrupts::enable_interrupts();
    log::trace!("\t* enabled interrupts");

    log::trace!("interrupts initialised");
}

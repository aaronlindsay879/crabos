use core::{arch::asm, fmt::Debug};

use bitflags::bitflags;
use lazy_static::lazy_static;
use x86_64::{
    registers::CR2,
    structures::{ExceptionStackFrame, InterruptDescriptorTable},
};

use crate::{gdt, println};

mod syscall;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

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

        idt[0x80].set(syscall_handler);

        idt
    };
}

extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: ExceptionStackFrame) {
    println!("\nEXCEPTION: DIVIDE BY ZERO\n{}", stack_frame);

    x86_64::hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: ExceptionStackFrame) {
    println!(
        "\nEXCEPTION: INVALID OPCODE at {:#x}\n{}",
        stack_frame.instruction_pointer, stack_frame
    );

    x86_64::hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: ExceptionStackFrame) {
    println!(
        "\nEXCEPTION: BREAKPOINT at {:#x}\n{}",
        stack_frame.instruction_pointer, stack_frame
    );
}

extern "x86-interrupt" fn double_fault(stack_frame: ExceptionStackFrame, err: u64) -> ! {
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
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:#x}\
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
    println!(
        "\nEXCEPTION: GENERAL PROTECTION FAULT while accessing {:#x}\
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
            "jmp [SYSCALL_TABLE + rax*8]",
            options(noreturn)
        );
    }
}

pub fn init() {
    IDT.load();
}

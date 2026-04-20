use crate::println;
use lazy_static::lazy_static;
use paste::paste;

mod idt;
mod stack_frame;

use idt::PageFaultErrorCode;
use stack_frame::InterruptStackFrame;

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.divide_error.set_handler(divide_by_zero_handler);
        idt.breakpoint.set_handler(breakpoint_handler);
        idt.invalid_opcode.set_handler(invalid_op_code_handler);
        idt.page_fault.set_handler(page_fault_handler);
        idt.double_fault.set_handler(double_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

macro_rules! handler {
    ($name: ident) => {
        paste! {
            extern "x86-interrupt" fn [<$name _handler>](
                stack_frame: stack_frame::InterruptStackFrame,
            ) {
                println!("EXCEPTION: {}\n{:#?}",stringify!([<$name:upper>]), stack_frame);
                loop {}
            }
        }
    };
}

handler!(divide_by_zero);
handler!(breakpoint);
handler!(invalid_op_code);

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    let accessed_address = Cr2::read();
    println!(
        "\nEXCEPTION: PAGE FAULT while accessing {:?}\
        \nerror code: {:?}\n{:#?}",
        accessed_address, error_code, stack_frame
    );
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn divide_by_zero() {
    unsafe {
        core::arch::asm!(
            "xor rdx, rdx",
            "mov rax, 1",
            "mov rcx, 0",
            "div rcx",
            out("rax") _,
            out("rdx") _,
            out("rcx") _,
        );
    }
}

#[test_case]
fn test_divide_by_zero_exception() {
    divide_by_zero();
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

#[test_case]
fn test_invalid_op_code_exception() {
    unsafe { core::arch::asm!("ud2") };
}

#[test_case]
fn test_page_fault_exception() {
    unsafe { *(0xdeadbeaf as *mut u64) = 42 };
}

use lazy_static::lazy_static;

use crate::println;
use x86_64::structures::idt::InterruptStackFrame;

mod idt;

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(3, breakpoint_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

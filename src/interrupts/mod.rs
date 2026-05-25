mod idt;
mod stack_frame;

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, hlt_loop, print, println};
use idt::{Idt, PageFaultErrorCode};
use pic8259::ChainedPics;
use stack_frame::InterruptStackFrame;

use lazy_static::lazy_static;
use paste::paste;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.divide_error.set_handler(divide_by_zero_handler);
        idt.breakpoint.set_handler(breakpoint_handler);
        idt.invalid_opcode.set_handler(invalid_op_code_handler);
        idt.page_fault.set_handler(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler(keyboard_interrupt_handler);
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
                stack_frame: InterruptStackFrame,
            ) {
                println!("EXCEPTION: {}\n{:#?}",stringify!([<$name:upper>]), stack_frame);
            }
        }
    };
}

handler!(divide_by_zero);
handler!(breakpoint);
handler!(invalid_op_code);

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    //print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    crate::multitasking::invoke_scheduler();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1, layouts};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    crate::multitasking::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

/*
#[test_case]
fn test_divide_by_zero_exception() {
    divide_by_zero();
}

#[test_case]
fn test_invalid_op_code_exception() {
    unsafe { core::arch::asm!("ud2") };
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
}*/

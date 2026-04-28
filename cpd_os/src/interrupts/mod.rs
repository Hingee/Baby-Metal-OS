pub mod gdt;
mod idt;
mod stack_frame;

pub fn init() {
    gdt::init();
    idt::init();
    unsafe { idt::PICS.lock().init() };
    x86_64::instructions::interrupts::enable();
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

/*
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
*/

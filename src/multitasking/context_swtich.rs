use super::Id;
use core::arch::{asm, global_asm};
use x86_64::VirtAddr;

global_asm!(include_str!("context_switch.s"));

pub unsafe fn context_switch_to(
    new_stack_pointer: VirtAddr,
    prev_thread_id: Id,
    switch_reason: SwitchReason,
) {
    asm!(
        "call asm_context_switch",
        in("rdi") new_stack_pointer.as_u64(),
        in("rsi") prev_thread_id.0,
        in("rdx") switch_reason as u64,
        lateout("rax") _, lateout("rcx") _, lateout("rdx") _,
        lateout("rsi") _, lateout("rdi") _,
        lateout("r8")  _, lateout("r9")  _, lateout("r10") _,
        lateout("r11") _,
        options(nostack),
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn add_paused_thread(
    paused_stack_pointer: VirtAddr,
    paused_thread_id: Id,
    switch_reason: SwitchReason,
) {
    with_scheduler(|s| s.add_paused_thread(paused_stack_pointer, paused_thread_id, switch_reason));
}

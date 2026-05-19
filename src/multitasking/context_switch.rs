use super::{Id, with_scheduler};
use core::arch::{asm, global_asm};
use x86_64::VirtAddr;

global_asm!(include_str!("context_switch.s"));

#[repr(u64)]
pub enum SwitchReason {
    Paused,
    Yield,
    Blocked,
    Exit,
}

/// # Safety
///
/// The caller must guarantee:
/// - `new_stack_pointer` points to a valid, mapped, writable stack.
/// - `new_stack_pointer` is 16-byte aligned.
/// - `prev_thread_id` identifies a thread that is currently registered with
///   the scheduler. Passing an unknown id will cause `add_paused_thread` to
///   panic with `ThreadNotFound`.
/// - This function must not be called while the scheduler lock is held on the
///   current core — `asm_context_switch` calls back into `add_paused_thread`
///   which calls `with_scheduler`, causing a deadlock.
/// - The current stack must remain valid until `asm_context_switch` has
///   switched `rsp` to `new_stack_pointer`. Freeing or unmapping the current
///   stack before this point is undefined behaviour.
pub unsafe fn context_switch_to(
    new_stack_pointer: VirtAddr,
    prev_thread_id: Id,
    switch_reason: SwitchReason,
) {
    unsafe {
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
}

#[unsafe(no_mangle)]
pub extern "C" fn add_paused_thread(
    paused_stack_pointer: VirtAddr,
    paused_thread_id: Id,
    switch_reason: SwitchReason,
) {
    with_scheduler(|s| {
        s.add_paused_thread(paused_stack_pointer, paused_thread_id, switch_reason)
            .expect("scheduler failed to add paused thread")
    });
}

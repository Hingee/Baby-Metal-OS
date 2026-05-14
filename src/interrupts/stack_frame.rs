use core::{fmt, ops::Deref};
use volatile::Volatile;
use x86_64::VirtAddr;

#[repr(C)]
pub struct InterruptStackFrame {
    value: InterruptStackFrameValue,
}

impl InterruptStackFrame {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub unsafe fn as_mut(&mut self) -> Volatile<&mut InterruptStackFrameValue> {
        Volatile::new(&mut self.value)
    }
}

impl Deref for InterruptStackFrame {
    type Target = InterruptStackFrameValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl fmt::Debug for InterruptStackFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct InterruptStackFrameValue {
    pub instruction_pointer: VirtAddr,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: VirtAddr,
    pub stack_segment: u64,
}

impl InterruptStackFrameValue {
    #[inline(always)]
    pub unsafe fn iretq(&self) -> ! {
        unsafe {
            core::arch::asm!(
                "push {stack_segment}",
                "push {new_stack_pointer}",
                "push {rflags}",
                "push {code_segment}",
                "push {new_instruction_pointer}",
                "iretq",
                rflags = in(reg) self.cpu_flags,
                new_instruction_pointer = in(reg) self.instruction_pointer.as_u64(),
                new_stack_pointer = in(reg) self.stack_pointer.as_u64(),
                code_segment = in(reg) self.code_segment,
                stack_segment = in(reg) self.stack_segment,
                options(noreturn)
            )
        }
    }
}

impl fmt::Debug for InterruptStackFrameValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u64);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }

        let mut s = f.debug_struct("InterruptStackFrame");
        s.field("instruction_pointer", &self.instruction_pointer);
        s.field("code_segment", &self.code_segment);
        s.field("cpu_flags", &Hex(self.cpu_flags));
        s.field("stack_pointer", &self.stack_pointer);
        s.field("stack_segment", &self.stack_segment);
        s.finish()
    }
}

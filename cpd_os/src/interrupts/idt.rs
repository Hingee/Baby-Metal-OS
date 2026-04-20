use crate::interrupts::stack_frame::InterruptStackFrame;
use bitflags::bitflags;
use core::marker::PhantomData;
use x86_64::{VirtAddr, registers::segmentation::Segment};

#[derive(Clone)]
#[repr(C)]
#[repr(align(16))]
pub struct Idt {
    /// The vector number of the `#DE` exception is 0.
    pub divide_error: Entry<HandlerFunc>,

    /// The vector number of the `#DB` exception is 1.
    pub debug: Entry<HandlerFunc>,

    /// The vector number of the NMI exception is 2.
    pub non_maskable_interrupt: Entry<HandlerFunc>,

    /// The vector number of the `#BP` exception is 3.
    pub breakpoint: Entry<HandlerFunc>,

    /// The vector number of the `#OF` exception is 4.
    pub overflow: Entry<HandlerFunc>,

    /// The vector number of the `#BR` exception is 5.
    pub bound_range_exceeded: Entry<HandlerFunc>,

    /// The vector number of the `#UD` exception is 6.
    pub invalid_opcode: Entry<HandlerFunc>,

    /// The vector number of the `#DF` exception is 8.
    pub double_fault: Entry<DivergingHandlerFuncWithErrCode>,

    /// The vector number of the `#PF` exception is 14.
    pub page_fault: Entry<PageFaultHandlerFunc>,
}

impl Idt {
    #[inline]
    pub fn new() -> Idt {
        Idt {
            divide_error: Entry::missing(),
            debug: Entry::missing(),
            non_maskable_interrupt: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range_exceeded: Entry::missing(),
            invalid_opcode: Entry::missing(),
            double_fault: Entry::missing(),
            page_fault: Entry::missing(),
        }
    }

    #[inline]
    pub fn load(&'static self) {
        unsafe { self.load_unsafe() }
    }

    #[inline]
    pub unsafe fn load_unsafe(&self) {
        use x86_64::instructions::tables::lidt;
        unsafe {
            lidt(&self.pointer());
        }
    }

    pub fn pointer(&self) -> x86_64::structures::DescriptorTablePointer {
        use core::mem::size_of;
        use x86_64::instructions::tables::DescriptorTablePointer;

        DescriptorTablePointer {
            base: VirtAddr::new(self as *const _ as u64),
            limit: (size_of::<Self>() - 1) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry<F> {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<T> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer_low == other.pointer_low
            && self.options == other.options
            && self.pointer_middle == other.pointer_middle
            && self.pointer_high == other.pointer_high
            && self.reserved == other.reserved
    }
}

impl<F> Entry<F> {
    #[inline]
    pub const fn missing() -> Self {
        Entry {
            gdt_selector: 0,
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }
    #[inline]
    fn set_handler_addr(&mut self, addr: u64) -> &mut EntryOptions {
        use x86_64::instructions::segmentation;

        self.pointer_low = addr as u16;
        self.pointer_middle = (addr >> 16) as u16;
        self.pointer_high = (addr >> 32) as u32;

        self.gdt_selector = segmentation::CS::get_reg().0;

        self.options.set_present(true);
        &mut self.options
    }
}

pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);
pub type PageFaultHandlerFunc =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErrorCode);
pub type DivergingHandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame) -> !;
pub type DivergingHandlerFuncWithErrCode =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64) -> !;

macro_rules! impl_set_handler {
    ($handler:ty) => {
        impl Entry<$handler> {
            #[inline]
            pub fn set_handler(&mut self, handler: $handler) -> &mut EntryOptions {
                self.set_handler_addr(handler as u64)
            }
        }
    };
}

impl_set_handler!(HandlerFunc);
impl_set_handler!(HandlerFuncWithErrCode);
impl_set_handler!(PageFaultHandlerFunc);
impl_set_handler!(DivergingHandlerFunc);
impl_set_handler!(DivergingHandlerFuncWithErrCode);

use bit_field::BitField;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntryOptions(u16);

impl EntryOptions {
    const fn minimal() -> Self {
        EntryOptions(0b1110_0000_0000)
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut Self {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut Self {
        self.0.set_bits(13..15, dpl);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut Self {
        self.0.set_bits(0..3, index);
        self
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct PageFaultErrorCode: u64 {
        const PROTECTION_VIOLATION = 1;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}

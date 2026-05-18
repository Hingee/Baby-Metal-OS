use super::{FrameAllocator, Size4KiB, VirtAddr};
use alloc::boxed::Box;
use core::{
    arch::naked_asm,
    mem,
    sync::atomic::{AtomicU64, Ordering},
};
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, mapper};

pub fn alloc_stack(
    size_in_pages: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<StackBounds, mapper::MapToError<Size4KiB>> {
    static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_5555_5555_0000);

    let guard_page_start = STACK_ALLOC_NEXT.fetch_add(
        (size_in_pages + 1) * Page::<Size4KiB>::SIZE,
        Ordering::SeqCst,
    );
    let guard_page = Page::<Size4KiB>::from_start_address(VirtAddr::new(guard_page_start))
        .expect("`STACK_ALLOC_NEXT` not page aligned");

    let stack_start = guard_page + 1;
    let stack_end = stack_start + size_in_pages;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in Page::range(stack_start, stack_end) {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(mapper::MapToError::FrameAllocationFailed)?;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    Ok(StackBounds {
        strt: stack_start.start_address(),
        end: stack_end.start_address(),
    })
}

pub struct Stack {
    ptr: VirtAddr,
}

impl Stack {
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarentee that the
    /// provided VirtAddr is valid, the allocation is writable, ptr is 16-byte aligned
    /// and is a unique pointer
    pub unsafe fn new(stack_pointer: VirtAddr) -> Self {
        Stack { ptr: stack_pointer }
    }

    pub fn get_stack_pointer(self) -> VirtAddr {
        self.ptr
    }

    pub fn set_up_for_closure(&mut self, closure: Box<dyn FnOnce()>) {
        let (data, vtable): (*mut (), *mut ()) = unsafe { mem::transmute(closure) };

        unsafe { self.push(data) };
        unsafe { self.push(vtable) };

        self.set_up_for_entry_point(call_closure_entry);
    }

    pub fn set_up_for_entry_point(&mut self, entry_point: extern "C" fn() -> ()) {
        unsafe { self.push(entry_point) };
        let rflags: u64 = 0x200;
        unsafe { self.push(rflags) };
    }

    unsafe fn push<T>(&mut self, value: T) {
        self.ptr -= core::mem::size_of::<T>();
        let ptr: *mut T = self.ptr.as_mut_ptr();
        unsafe { ptr.write(value) };
    }
}

#[unsafe(naked)]
extern "C" fn call_closure_entry() {
    naked_asm!(
        "pop rsi", //
        "pop rdi", //
        "jmp call_closure",
    )
}

#[unsafe(no_mangle)]
extern "C" fn call_closure(data: *mut (), vtable: *mut ()) -> ! {
    let f: Box<dyn FnOnce()> = unsafe { mem::transmute((data, vtable)) };
    f();
    unreachable!();
}

pub struct StackBounds {
    strt: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub fn start(&self) -> VirtAddr {
        self.strt
    }

    pub fn end(&self) -> VirtAddr {
        self.end
    }
}

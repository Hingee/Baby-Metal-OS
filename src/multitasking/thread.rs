use super::Id;
use crate::memory::stack::{Stack, StackBounds, alloc_stack};
use alloc::boxed::Box;
use core::sync::atomic::Ordering;
use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, Mapper, Size4KiB, mapper},
};

#[derive(Debug)]
pub enum ThreadError {
    ThreadAlreadyExists,
    IdleThreadAlreadySet,
    ThreadNotFound,
    StackPointerAlreadyPresent,
    StackBoundsNotFound,
    StackAllocationFailed(mapper::MapToError<Size4KiB>),
}

impl From<mapper::MapToError<Size4KiB>> for ThreadError {
    fn from(e: mapper::MapToError<Size4KiB>) -> Self {
        ThreadError::StackAllocationFailed(e)
    }
}

pub struct Thread {
    id: Id,
    stack_pointer: Option<VirtAddr>,
    _stack_bounds: Option<StackBounds>,
}

impl Thread {
    pub fn create(
        entry_point: extern "C" fn() -> !,
        stack_size: u64,
        mapper: &mut impl Mapper<Size4KiB>,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> Result<Self, ThreadError> {
        let stack_bounds = alloc_stack(stack_size, mapper, frame_allocator)?;
        let mut stack = unsafe { Stack::new(stack_bounds.end()) };
        stack.set_up_for_entry_point(entry_point);
        Ok(Self::new(stack.get_stack_pointer(), stack_bounds))
    }

    pub fn create_from_closure<F>(
        closure: F,
        stack_size: u64,
        mapper: &mut impl Mapper<Size4KiB>,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    ) -> Result<Self, ThreadError>
    where
        F: FnOnce() + 'static + Send + Sync,
    {
        let stack_bounds = alloc_stack(stack_size, mapper, frame_allocator)?;
        let mut stack = unsafe { Stack::new(stack_bounds.end()) };
        stack.set_up_for_closure(Box::new(closure));
        Ok(Self::new(stack.get_stack_pointer(), stack_bounds))
    }

    pub fn new(stack_pointer: VirtAddr, stack_bounds: StackBounds) -> Thread {
        Thread {
            id: Id::new(Ordering::SeqCst),
            stack_pointer: Some(stack_pointer),
            _stack_bounds: Some(stack_bounds),
        }
    }

    pub(super) fn create_root_thread() -> Self {
        Thread {
            id: Id(0),
            stack_pointer: None,
            _stack_bounds: None,
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub(super) fn stack_pointer(&mut self) -> &mut Option<VirtAddr> {
        &mut self.stack_pointer
    }
}

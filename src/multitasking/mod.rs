use crate::memory::stack::StackBounds;
use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};
use x86_64::VirtAddr;

pub mod context_swtich;
pub mod executor;
pub mod keyboard;

const QUEUE_MAX: usize = 100;

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
struct Id(u64);

impl Id {
    fn new(counter: AtomicU64, ordering_type: Ordering) -> Self {
        Id(counter.fetch_add(1, ordering_type))
    }
}

pub struct Task {
    id: Id,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: Id::new(AtomicU64::new(1), Ordering::Relaxed),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

pub struct Thread {
    id: Id,
    stack_pointer: Option<VirtAddr>,
    stack_bounds: Option<StackBounds>,
}

impl Thread {
    pub fn new(stack_pointer: VirtAddr, stack_bounds: StackBounds) -> Thread {
        Thread {
            id: Id::new(AtomicU64::new(1), Ordering::SeqCst),
            stack_pointer: Some(stack_pointer),
            stack_bounds: Some(stack_bounds),
        }
    }
}

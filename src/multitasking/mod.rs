use core::sync::atomic::{AtomicU64, Ordering};
use scheduler::Scheduler;
use spin::Mutex;

pub mod context_switch;
pub mod executor;
pub mod keyboard;
pub mod scheduler;
pub mod task;
pub mod thread;

const QUEUE_MAX: usize = 100;

#[repr(transparent)]
#[derive(Ord, Eq, PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct Id(u64);

impl Id {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    fn new(ordering_type: Ordering) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Id(NEXT_ID.fetch_add(1, ordering_type))
    }
}

static SCHEDULER: Mutex<Option<Scheduler>> = spin::Mutex::new(None);

pub fn invoke_scheduler() {
    let next = SCHEDULER
        .try_lock()
        .and_then(|mut scheduler| scheduler.as_mut().and_then(|s| s.schedule()));
    if let Some((next_stack_pointer, prev_thread_id)) = next {
        unsafe {
            context_switch::context_switch_to(
                next_stack_pointer,
                prev_thread_id,
                context_switch::SwitchReason::Paused,
            )
        };
    }
}

pub fn exit_thread() -> ! {
    synchronous_context_switch(context_switch::SwitchReason::Exit).expect("can't exit last thread");
    unreachable!("finished thread continued");
}

pub fn yield_now() {
    let _ = synchronous_context_switch(context_switch::SwitchReason::Yield);
}

fn synchronous_context_switch(reason: context_switch::SwitchReason) -> Result<(), ()> {
    let next = with_scheduler(|s| s.schedule());
    match next {
        Some((next_stack_pointer, prev_thread_id)) => unsafe {
            context_switch::context_switch_to(next_stack_pointer, prev_thread_id, reason);
            Ok(())
        },
        None => Err(()),
    }
}

pub fn with_scheduler<F, T>(f: F) -> T
where
    F: FnOnce(&mut Scheduler) -> T,
{
    f(SCHEDULER.lock().get_or_insert_with(Scheduler::new))
}

use super::{
    Id,
    context_switch::SwitchReason,
    thread::{Thread, ThreadError},
};
use alloc::collections::{BTreeMap, BTreeSet, VecDeque, btree_map::Entry};
use core::mem;
use x86_64::VirtAddr;

pub struct Scheduler {
    threads: BTreeMap<Id, Thread>,
    idle_thread_id: Option<Id>,
    current_thread_id: Id,
    paused_threads: VecDeque<Id>,
    blocked_threads: BTreeSet<Id>,
    wakeups: BTreeSet<Id>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let root_thread = Thread::create_root_thread();
        let root_id = root_thread.id();
        let mut threads = BTreeMap::new();
        let old_value = threads.insert(root_id, root_thread);
        assert!(
            old_value.is_none(),
            "Static analysis guarantee: map must be empty"
        );

        Scheduler {
            threads,
            idle_thread_id: None,
            current_thread_id: root_id,
            paused_threads: VecDeque::new(),
            blocked_threads: BTreeSet::new(),
            wakeups: BTreeSet::new(),
        }
    }

    fn next_thread(&mut self) -> Option<Id> {
        self.paused_threads.pop_front()
    }

    pub fn add_new_thread(&mut self, thread: Thread) -> Result<(), ThreadError> {
        let thread_id = thread.id();

        match self.threads.entry(thread_id) {
            Entry::Vacant(entry) => {
                entry.insert(thread);
            }
            Entry::Occupied(_) => {
                return Err(ThreadError::ThreadAlreadyExists);
            }
        }

        self.paused_threads.push_back(thread_id);
        Ok(())
    }

    pub fn set_idle_thread(&mut self, thread: Thread) -> Result<(), ThreadError> {
        let thread_id = thread.id();

        if self.idle_thread_id.is_some() {
            return Err(ThreadError::IdleThreadAlreadySet);
        }

        match self.threads.entry(thread_id) {
            Entry::Vacant(entry) => {
                entry.insert(thread);
            }
            Entry::Occupied(_) => {
                return Err(ThreadError::ThreadAlreadyExists);
            }
        }

        self.idle_thread_id = Some(thread_id);
        Ok(())
    }

    pub(super) fn add_paused_thread(
        &mut self,
        paused_stack_pointer: VirtAddr,
        paused_thread_id: Id,
        switch_reason: SwitchReason,
    ) -> Result<(), ThreadError> {
        let paused_thread = self
            .threads
            .get_mut(&paused_thread_id)
            .ok_or(ThreadError::ThreadNotFound)?;

        if paused_thread
            .stack_pointer()
            .replace(paused_stack_pointer)
            .is_some()
        {
            return Err(ThreadError::StackPointerAlreadyPresent);
        }

        if Some(paused_thread_id) == self.idle_thread_id {
            return Ok(());
        }

        match switch_reason {
            SwitchReason::Paused | SwitchReason::Yield => {
                self.paused_threads.push_back(paused_thread_id);
            }
            SwitchReason::Blocked => {
                self.blocked_threads.insert(paused_thread_id);
                self.check_for_wakeup(paused_thread_id);
            }
            SwitchReason::Exit => {
                let _thread = self
                    .threads
                    .remove(&paused_thread_id)
                    .ok_or(ThreadError::ThreadNotFound)?;
                // TODO: free stack memory again
                //crate::memory::stack::dealloc_stack(&thread.stack_bounds, mapper, frame_allocator);
            }
        }

        Ok(())
    }

    fn check_for_wakeup(&mut self, thread_id: Id) {
        if self.wakeups.remove(&thread_id) {
            assert!(self.blocked_threads.remove(&thread_id));
            self.paused_threads.push_back(thread_id);
        }
    }

    pub fn schedule(&mut self) -> Option<(VirtAddr, Id)> {
        let mut next_thread_id = self.next_thread();
        if next_thread_id.is_none() && Some(self.current_thread_id) != self.idle_thread_id {
            next_thread_id = self.idle_thread_id
        }
        if let Some(next_id) = next_thread_id {
            let next_thread = self
                .threads
                .get_mut(&next_id)
                .expect("next thread does not exist");
            let next_stack_pointer = next_thread
                .stack_pointer()
                .take()
                .expect("paused thread has no stack pointer");
            let prev_thread_id = mem::replace(&mut self.current_thread_id, next_thread.id());
            Some((next_stack_pointer, prev_thread_id))
        } else {
            None
        }
    }

    pub fn current_thread_id(&self) -> Id {
        self.current_thread_id
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

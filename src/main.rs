#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(baby_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use baby_os::{
    allocator,
    memory::{self, BootInfoFrameAllocator},
    multitasking,
    multitasking::{executor::Executor, keyboard, task::Task, with_scheduler},
    print, println,
};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    baby_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap Init Failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    /*let idle_thread =
        multitasking::thread::Thread::create(idle_thread, 2, &mut mapper, &mut frame_allocator)
            .unwrap();
    with_scheduler(|s| {
        s.set_idle_thread(idle_thread)
            .expect("failed to set idle thread")
    });

    for _ in 0..10 {
        let thread = multitasking::thread::Thread::create(
            thread_entry,
            2,
            &mut mapper,
            &mut frame_allocator,
        )
        .unwrap();
        with_scheduler(|s| s.add_new_thread(thread).expect("failed to add thread"));
    }

    let thread = multitasking::thread::Thread::create_from_closure(
        || thread_entry(),
        2,
        &mut mapper,
        &mut frame_allocator,
    )
    .unwrap();
    with_scheduler(|s| {
        s.add_new_thread(thread)
            .expect("failed to add closure thread")
    });

    println!("It did not crash!");
    thread_entry();*/
}

#[allow(dead_code)]
async fn async_number() -> u32 {
    42
}

#[allow(dead_code)]
async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

#[allow(dead_code)]
extern "C" fn idle_thread() -> ! {
    loop {
        x86_64::instructions::hlt();
        multitasking::yield_now();
    }
}

#[allow(dead_code)]
extern "C" fn thread_entry() -> ! {
    let thread_id = with_scheduler(|s| s.current_thread_id()).as_u64();
    for _ in 0..=thread_id {
        print!("{}", thread_id);
        x86_64::instructions::hlt();
    }
    multitasking::exit_thread();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    baby_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    baby_os::test_panic_handler(info);
}

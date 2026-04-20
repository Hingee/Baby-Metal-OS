#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(cpd_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use cpd_os::println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    cpd_os::init();

    //cpd_os::interrupts::divide_by_zero();
    //x86_64::instructions::interrupts::int3(); //breakpoint
    //unsafe { core::arch::asm!("ud2") }; //invalid op-code
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    }; //page-fault

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cpd_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

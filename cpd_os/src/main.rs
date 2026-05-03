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

    use x86_64::registers::control::Cr3;

    let (lvl_4_page_table, _) = Cr3::read();
    println!(
        "Level 4 page table at: {:?}",
        lvl_4_page_table.start_address()
    );

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    cpd_os::hlt_loop();
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
    cpd_os::test_panic_handler(info);
    cpd_os::hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

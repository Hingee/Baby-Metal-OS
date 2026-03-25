#![no_std]
#![no_main]

use core::panic::PanicInfo;

//static has actual mem location not inlined
static HELLO: &[u8] = b"Hello World!";

//const has NO actual mem location is inlined
const VGA_BUFFER_STRT: u32 = 0xb8000;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let vga_buf = VGA_BUFFER_STRT as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buf.offset(i as isize * 2) = byte;
            *vga_buf.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

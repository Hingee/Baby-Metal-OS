## Layer 2 - VGA

The VGA text mode is a simple, memory-mapped I/O method used to display text on the screen before a full graphical driver is implemented.

### 1. The VGA Text Buffer Structure

The buffer is a 2D array (typically **25 rows × 80 columns**) located at the physical memory address **0xb8000**. Each entry in the array consists of 16 bits (2 bytes):

* **Bits 0-7:** ASCII code point (using Code Page 437).
* **Bits 8-11:** Foreground color.
* **Bits 12-14:** Background color.
* **Bit 15:** Blink bit.

### 2. Implementation in Rust

To make the buffer safe and "Rust-like," the implementation is encapsulated in a module (e.g., `vga_buffer.rs`):

* **Color Representation:** Uses a C-like `enum` with `#[repr(u8)]` to map colors to their hardware-defined values (0-15).
* **Memory Layout:** Structs like `ScreenChar` and `Buffer` use `#[repr(C)]` or `#[repr(transparent)]` to ensure the memory layout matches exactly what the VGA hardware expects.
* **Volatile Access:** Because the compiler might optimize away "unused" writes to the buffer (since we never read from it), the `volatile` crate is used. This ensures every write actually reaches the hardware.

### 3. The Global Interface & Synchronization

Since we don't have a standard library or OS-level mutexes, creating a global `WRITER` requires specific tools:

* **`lazy_static`:** Used to initialize the `WRITER` at runtime rather than compile time, allowing for complex setup like pointer dereferencing.
* **Spinlocks:** Since we lack thread blocking, a `spin::Mutex` is used for synchronization. If the `WRITER` is locked, the CPU simply "spins" in a loop until it becomes available.

### 4. Integration with Rust Features

* **Formatting Macros:** By implementing the `core::fmt::Write` trait for the `Writer` struct, the kernel can use Rust’s standard `write!` and `writeln!` macros.
* **Custom `println!`:** We define our own `print!` and `println!` macros using `#[macro_export]`. These call a hidden `_print` function that locks the global `WRITER`.
* **Panic Handling:** Once the `println!` macro is functional, it is integrated into the `#[panic_handler]` to print the panic message and source code location directly to the screen.

---

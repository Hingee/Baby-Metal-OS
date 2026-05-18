## Layer 1 - Booting & Freestanding Setup

### Creating a Freestanding Rust Binary

#### 1. Disabling the Standard Library (`no_std`)

By default, Rust links the **Standard Library (`std`)**, which relies on OS features (threads, files, networking). To build an OS, you must disable this.

- **The Attribute:** Use `#![no_std]` at the top of your `main.rs`.
- **What stays:** You can still use "pure" Rust features like Iterators, Closures, Pattern Matching, Option/Result, and the Ownership system.
- **What goes:** You lose `println!`, `Vec`, `String` (heap-allocated), and anything requiring a file system or network.

#### 2. Handling Panics

Without the standard library, the compiler doesn't know what to do if the code "panics" (crashes).

- **Panic Handler:** You must define a function with the `#[panic_handler]` attribute.
- **The Signature:** It takes `&PanicInfo` and returns `!`, the "never" type. This means the function is "diverging" (it never returns; it usually just loops forever).

#### 3. Disabling Unwinding

Normally, Rust "unwinds the stack" on panic to clean up memory. This requires OS-specific libraries (like `libunwind`).

- **The Fix:** Change the panic strategy to **abort** in your `Cargo.toml`. This simplifies the binary and removes the need for the `eh_personality` language item.

### A Minimal Rust Kernel

#### 1. Boot Process

When a computer turns on it begins executing firmware code that is stored in motherboard ROM. This firmware performs a power-on self-test which detects available RAM and initializes CPU and hardware. Afterwards, it looks for a bootable disk and starts booting the operating system kernel.

x86 has 2 different firmware standards:

- **(BIOS):** `Basic Input/Output System` -- **SELECTED STANDARD FOR THIS CPD**
- **(UEFI):** `Unified Extensible Firmware Interface`

#### 2. BIOS Boot

**BIOS** has support on almost all x86 systems and some newer **UEFI** use an emulated **BIOS**.

- **Pro:** Widely compatible.
- **Con:** BIOS booting puts the CPU into 16-bit compatibility mode called

After the normal boot process when BIOS finds a bootable disk control is transferred to its bootloader (a 512-byte portion of executable code stored ate the disks beginning.Most bootloaders are larger than 512 bytes, so bootloaders are commonly split into a 512-byte first stage, and a second stage which is loaded by the first stage.

The bootloader has to determine the kernel location on the disk, load it into memory, switch the CPU from `16-bit real mode` -> `32-bit protected mode` -> `64-bit long mode`, and query other information from the BIOS and pass it to the OS kernel.

#### 3. Custom Target (`.json`)

Since I am building for "bare metal" and not on an OS. I needed to setup a custom target using a `JSON` file to tell `rustc` how to build for machine code.

- `"os": "none"`: Tells the compiler to not use any OS-specific syscalls or libraries (like libc).
- **The LLD Linker:** `rust-lld` is used as a cross-platform linker shipped with Rust. It doesn't rely on the host system's default linker.

- **Disabling the "Red Zone":** The "Red Zone" is an optimization in the x86_64 System V ABI that allows functions to temporarily use the 128 bytes below their stack frame without adjusting the stack pointer. Since in layer 3 I am going to want to handle interrupts I will disable this safety feature.

#### 4. Floating Point & SIMD

In kernel space, I am avoiding support for SIMD (Single Instruction Multiple Data) instructions. This is because the large SIMD registers in OS kernels lead to performance problems as a kernel needs to restore all registers to their original state before continuing an interrupted program. Hence each time this happens the kernel would need to save the COMPLETE SIMD state (512-1600 bytes) to main memory.

#### 5. Rebuilding Core (`build-std`)

Because the custom target is unique, the pre-compiled core library provided by `rustup` won't work. Hence I use `build-std` a nightly feature which tells Cargo to compile core and compiler_builtins from my `.json` target. By also enabling `compiler-builtins-mem`, we get Rust-native versions of memset, memcpy, and memcmp. Which I will use as they are safe and tested implementations of these methods.

#### 6. The VGA Text Buffer

The simplest way to verify the kernel is by writing directly to the VGA Text Buffer.

- **Memory Mapping:** The VGA buffer is a memory-mapped I/O region starting at physical address 0xb8000.

- **The Layout:** It is an array of roughly 25×80 characters.

#### 7. The Bootimage Tool

To make the kernel bootable, the `bootimage` tool glues the compiled ELF binary to a bootloader.

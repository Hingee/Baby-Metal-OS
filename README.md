# Baby Metal OS

Baby Metal OS is a bare-metal x86_64 kernel written in freestanding Rust. This project explores low-level systems programming, focusing on kernel development, memory management, and hardware abstraction without the standard library.

## Prerequisites

To build and run this project, you will need:

* **Rust Nightly:** The project uses experimental features available only in the nightly channel.
    ```bash
    rustup override set nightly
    ```
* **llvm-tools-preview:** Required for building the bootable image.
    ```bash
    rustup component add llvm-tools-preview
    ```
* **bootimage:** A tool to create a bootable disk image from the kernel.
    ```bash
    cargo install bootimage
    ```
* **QEMU:** Required to emulate the x86_64 hardware and run the kernel.

## Getting Started

The project includes a helper script to manage builds, updates, and emulation.

### Basic Usage

You can view all available commands by running:
```bash
./run.sh -h

//! ParyantaOS Exokernel
//!
//! The minimal kernel that handles:
//! - CPU time allocation
//! - Physical memory page ownership
//! - Device access tokens
//!
//! All other OS functionality lives in the Library OS crates.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod memory;
mod serial;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;

/// Configure bootloader to map all physical memory.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

// Register kernel entry point with config.
entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// Kernel entry point — called by bootloader after setting up paging.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialize serial output first (for debugging in QEMU)
    serial::init();

    println!();
    println!("=====================================");
    println!("  ParyantaOS Exokernel v0.1.0");
    println!("  \"The Compiler is the Kernel\"");
    println!("=====================================");
    println!();

    // Initialize CPU (GDT, IDT, TSS)
    arch::init();
    println!("[OK] GDT initialized");
    println!("[OK] IDT initialized");
    println!("[OK] TSS initialized");
    println!();

    // Initialize memory subsystem (frame allocator + heap)
    memory::init(boot_info);

    // Prove that alloc works!
    {
        use alloc::vec::Vec;
        let mut v = Vec::new();
        for i in 0..5 {
            v.push(i + 1);
        }
        println!("[OK] alloc works! vec = {:?}", v);
    }

    // Prove that Box works!
    {
        use alloc::boxed::Box;
        let boxed = Box::new(42u64);
        println!("[OK] Box works! boxed = {}", boxed);
    }

    println!();
    println!("Exokernel ready. Halting CPU.");
    println!("Press Ctrl+A, X to exit QEMU.");

    halt_loop();
}

/// Halt the CPU forever (low power).
pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler for kernel panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);

    halt_loop();
}

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

mod arch;
mod serial;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

// Register kernel entry point via bootloader_api macro.
entry_point!(kernel_main);

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

    // Print memory map from bootloader
    let memory_regions = &boot_info.memory_regions;
    println!("Memory regions from bootloader:");
    for (i, region) in memory_regions.iter().enumerate().take(5) {
        println!("  Region {}: {:#x} - {:#x} ({:?})",
            i, region.start, region.end, region.kind);
    }
    if memory_regions.len() > 5 {
        println!("  ... and {} more regions", memory_regions.len() - 5);
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

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
mod task;

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

    // Initialize CPU (GDT, IDT, TSS, PIC, enable interrupts)
    arch::init();
    println!("[OK] GDT initialized");
    println!("[OK] IDT initialized");
    println!("[OK] TSS initialized");
    println!("[OK] PIC initialized (IRQs 32-47)");
    println!("[OK] Interrupts enabled");
    println!();

    // Initialize memory subsystem (frame allocator + heap)
    memory::init(boot_info);

    // Quick alloc sanity check
    {
        use alloc::vec;
        let v = vec![1, 2, 3, 4, 5];
        println!("[OK] alloc works! vec = {:?}", v);
    }

    println!();

    // ── Scheduler demo ─────────────────────────────────────────
    let mut sched = task::scheduler::Scheduler::new();

    // TaskA: prints a message on each step
    sched.spawn("TaskA", 5, |step| {
        println!("  [TaskA] step {}", step);
    });

    // TaskB: prints a message on each step
    sched.spawn("TaskB", 5, |step| {
        println!("  [TaskB] step {}", step);
    });

    sched.run();

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
    // Disable interrupts during panic to prevent re-entrancy
    x86_64::instructions::interrupts::disable();

    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);

    loop {
        x86_64::instructions::hlt();
    }
}

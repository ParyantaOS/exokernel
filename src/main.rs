//! ParyantaOS Exokernel
//! 
//! The minimal kernel that handles:
//! - CPU time allocation
//! - Physical memory page ownership
//! - Device access tokens
//!
//! All other OS functionality lives in libparyanta (Library OS).

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod memory;
mod serial;

use core::panic::PanicInfo;

/// Entry point after bootloader hands off control.
/// 
/// # Safety
/// This function is called by the bootloader and must never return.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize serial output first (for debugging in QEMU)
    serial::init();
    
    println!("ParyantaOS Exokernel v0.1.0");
    println!("===========================");
    
    // Initialize CPU (GDT, IDT, TSS)
    arch::init();
    println!("[OK] CPU initialized");
    
    // Initialize memory management
    memory::init();
    println!("[OK] Memory initialized");
    
    // Hand off to Library OS
    println!("Jumping to Library OS...");
    
    // TODO: Jump to libparyanta::main()
    // For now, just halt
    println!("Exokernel ready. Halting.");
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler for kernel panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n!!! KERNEL PANIC !!!");
    println!("{}", info);
    
    loop {
        x86_64::instructions::hlt();
    }
}

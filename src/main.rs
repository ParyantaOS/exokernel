//! ParyantaOS Exokernel
//!
//! The minimal kernel that handles:
//! - CPU time allocation
//! - Physical memory page ownership
//! - Device access tokens
//! - Capability-based security
//!
//! All other OS functionality lives in the Library OS crates.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod caps;
mod memory;
mod serial;
mod task;

use alloc::vec;
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use caps::{Rights, Resource};
use caps::manager as cap_mgr;
use core::panic::PanicInfo;

/// Configure bootloader to map all physical memory.
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// Kernel entry point — called by bootloader after setting up paging.
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
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

    // Initialize memory subsystem
    memory::init(boot_info);
    println!();

    // ── Capability System Demo ────────────────────────────────
    println!("=== Capability System Demo ===");
    println!();

    // Mint capabilities (kernel-only operation)
    let mem_cap = cap_mgr::mint(
        Resource::Memory { base: 0x1000, size: 4096 },
        Rights::RW,
        true, // delegatable
    );
    println!("[CAP] Minted {} (Memory 0x1000, RW, delegatable)", mem_cap);

    let dev_cap = cap_mgr::mint(
        Resource::Device(0x3F8), // COM1
        Rights::READ,
        false, // not delegatable
    );
    println!("[CAP] Minted {} (Device COM1, R, non-delegatable)", dev_cap);
    println!();

    // Spawn tasks with different capabilities
    let mut sched = task::scheduler::Scheduler::new();

    // TaskA: holds mem_cap — should succeed
    sched.spawn("TaskA", 3, vec![mem_cap], |step, caps| {
        match step {
            0 => {
                // Try READ on our cap — should succeed
                let result = cap_mgr::verify(caps[0], Rights::READ);
                println!("  [TaskA] verify {} READ  → {}", caps[0],
                    if result.is_ok() { "✓ granted" } else { "✗ denied" });
            }
            1 => {
                // Try WRITE on our cap — should succeed
                let result = cap_mgr::verify(caps[0], Rights::WRITE);
                println!("  [TaskA] verify {} WRITE → {}", caps[0],
                    if result.is_ok() { "✓ granted" } else { "✗ denied" });
            }
            2 => {
                // Try EXECUTE on our cap — should fail (we only have RW)
                let result = cap_mgr::verify(caps[0], Rights::EXECUTE);
                println!("  [TaskA] verify {} EXEC  → {}", caps[0],
                    if result.is_ok() { "✓ granted" } else { "✗ denied (no EXEC right)" });
            }
            _ => {}
        }
    });

    // TaskB: holds NO caps — everything should be denied
    sched.spawn("TaskB", 2, vec![], |step, _caps| {
        match step {
            0 => {
                // Try to use mem_cap without holding it
                // (We reference cap#1 directly — but in a real system, tasks
                //  can only use caps they hold. Here we demonstrate the check.)
                println!("  [TaskB] has ZERO capabilities (zero ambient authority)");
                println!("  [TaskB] cannot access any resource without caps");
            }
            1 => {
                println!("  [TaskB] this is the exokernel security model:");
                println!("  [TaskB] \"no cap = no access\"");
            }
            _ => {}
        }
    });

    sched.run();

    // ── Restrict + Revoke Demo ────────────────────────────────
    println!();
    println!("=== Restrict + Revoke Demo ===");
    println!();

    // Restrict: RW → R only
    match cap_mgr::restrict(mem_cap, Rights::READ) {
        Ok(restricted) => {
            println!("[CAP] Restricted {} → {} (READ only)", mem_cap, restricted);

            // Verify READ on restricted cap — should work
            let r = cap_mgr::verify(restricted, Rights::READ);
            println!("  verify {} READ  → {}",
                restricted, if r.is_ok() { "✓ granted" } else { "✗ denied" });

            // Verify WRITE on restricted cap — should fail
            let r = cap_mgr::verify(restricted, Rights::WRITE);
            println!("  verify {} WRITE → {}",
                restricted, if r.is_ok() { "✓ granted" } else { "✗ denied (right not granted)" });
        }
        Err(e) => println!("[CAP] restrict failed: {}", e),
    }

    // Revoke the original mem_cap
    println!();
    match cap_mgr::revoke(mem_cap) {
        Ok(()) => {
            println!("[CAP] Revoked {}", mem_cap);

            // Verify on revoked cap — should fail
            let r = cap_mgr::verify(mem_cap, Rights::READ);
            println!("  verify {} READ  → {}",
                mem_cap, if r.is_ok() { "✓ granted" } else { "✗ denied (revoked)" });
        }
        Err(e) => println!("[CAP] revoke failed: {}", e),
    }

    // Try to restrict a non-delegatable cap
    println!();
    match cap_mgr::restrict(dev_cap, Rights::READ) {
        Ok(c) => println!("[CAP] restrict {} → {} (unexpected!)", dev_cap, c),
        Err(e) => println!("[CAP] restrict {} → ✗ {} (non-delegatable)", dev_cap, e),
    }

    println!();
    println!("=== Capability Demo Complete ===");
    println!();
    println!("Exokernel ready. Halting CPU.");

    halt_loop();
}

/// Halt the CPU forever (low power).
pub fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Panic handler.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);
    loop { x86_64::instructions::hlt(); }
}

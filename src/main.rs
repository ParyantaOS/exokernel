//! ParyantaOS Exokernel
//!
//! The minimal kernel that handles:
//! - CPU time allocation
//! - Physical memory page ownership
//! - Device access tokens
//! - Capability-based security
//! - Object Store ("Everything is a Database")
//!
//! All other OS functionality lives in the Library OS crates.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod arch;
mod caps;
mod memory;
mod objstore;
mod serial;
mod task;

use alloc::vec;
use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use caps::{Rights, Resource};
use caps::manager as cap_mgr;
use objstore::{Object, gated as obj};
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
    println!("[OK] GDT, IDT, TSS, PIC initialized");
    println!("[OK] Interrupts enabled");
    println!();

    // Initialize memory subsystem
    memory::init(boot_info);
    println!();

    // ── Capability System ─────────────────────────────────────
    println!("=== Capability System ===");
    println!();

    let rw_cap = cap_mgr::mint(Resource::Object(0), Rights::RW, true);
    println!("[CAP] Minted {} (Object Store, RW)", rw_cap);

    let r_cap = cap_mgr::mint(Resource::Object(0), Rights::READ, false);
    println!("[CAP] Minted {} (Object Store, R)", r_cap);

    let rwd_cap = cap_mgr::mint(
        Resource::Object(0),
        Rights::READ | Rights::WRITE | Rights::DELETE,
        false,
    );
    println!("[CAP] Minted {} (Object Store, RWD)", rwd_cap);
    println!();

    // ── Object Store Demo ─────────────────────────────────────
    println!("=== Object Store Demo ===");
    println!("\"Everything is a Database\"");
    println!();

    // Create objects with tags using RW cap
    let obj1 = Object::new(b"hello")
        .with_tag("greeting")
        .with_meta("lang", "en");

    match obj::create(rw_cap, obj1) {
        Ok(id) => println!("[STORE] Created {} (\"hello\", tag:greeting)", id),
        Err(e) => println!("[STORE] create failed: {}", e),
    }

    let obj2 = Object::new(b"hola mundo!")
        .with_tag("greeting")
        .with_tag("important")
        .with_meta("lang", "es");

    match obj::create(rw_cap, obj2) {
        Ok(id) => println!("[STORE] Created {} (\"hola mundo!\", tags:greeting,important)", id),
        Err(e) => println!("[STORE] create failed: {}", e),
    }

    let obj3 = Object::new(b"system config v1")
        .with_tag("config")
        .with_meta("version", "1");

    match obj::create(rw_cap, obj3) {
        Ok(id) => println!("[STORE] Created {} (\"system config v1\", tag:config)", id),
        Err(e) => println!("[STORE] create failed: {}", e),
    }

    println!();
    println!("[STORE] count: {} objects", objstore::store::count());
    println!();

    // ── Tag Queries ───────────────────────────────────────────
    println!("--- Tag Queries ---");
    match obj::query_by_tag(r_cap, "greeting") {
        Ok(ids) => {
            println!("[QUERY] tag:\"greeting\" → {} results", ids.len());
            for id in &ids {
                if let Ok(o) = obj::read(r_cap, *id) {
                    let text = core::str::from_utf8(&o.content).unwrap_or("(bin)");
                    println!("  {} → \"{}\"", id, text);
                }
            }
        }
        Err(e) => println!("[QUERY] failed: {}", e),
    }

    match obj::query_by_tag(r_cap, "config") {
        Ok(ids) => {
            println!("[QUERY] tag:\"config\"   → {} results", ids.len());
            for id in &ids {
                if let Ok(o) = obj::read(r_cap, *id) {
                    let text = core::str::from_utf8(&o.content).unwrap_or("(bin)");
                    println!("  {} → \"{}\"", id, text);
                }
            }
        }
        Err(e) => println!("[QUERY] failed: {}", e),
    }
    println!();

    // ── Access Control ────────────────────────────────────────
    println!("--- Access Control ---");

    // Try to create with READ-only cap → should fail
    let obj4 = Object::new(b"sneaky write");
    match obj::create(r_cap, obj4) {
        Ok(id) => println!("[STORE] create with R cap → {} (unexpected!)", id),
        Err(e) => println!("[STORE] create with R cap → ✗ {} (correct!)", e),
    }

    // Read with READ-only cap → should succeed
    let hello_id = objstore::ObjId::from_content(b"hello");
    match obj::read(r_cap, hello_id) {
        Ok(o) => {
            let text = core::str::from_utf8(&o.content).unwrap_or("(bin)");
            println!("[STORE] read with R cap → ✓ \"{}\"", text);
        }
        Err(e) => println!("[STORE] read failed: {}", e),
    }

    // Delete with RW cap (no DELETE right) → should fail
    match obj::delete(rw_cap, hello_id) {
        Ok(()) => println!("[STORE] delete with RW cap  → deleted (unexpected!)"),
        Err(e) => println!("[STORE] delete with RW cap  → ✗ {} (no DELETE right)", e),
    }

    // Delete with RWD cap → should succeed
    match obj::delete(rwd_cap, hello_id) {
        Ok(()) => println!("[STORE] delete with RWD cap → ✓ deleted"),
        Err(e) => println!("[STORE] delete with RWD cap → failed: {}", e),
    }

    println!();
    println!("[STORE] count: {} objects (after delete)", objstore::store::count());

    println!();
    println!("=== Object Store Demo Complete ===");
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

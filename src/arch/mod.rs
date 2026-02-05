//! x86_64 architecture-specific initialization.

mod gdt;
mod idt;

/// Initialize CPU structures (GDT, IDT).
pub fn init() {
    gdt::init();
    idt::init();
}

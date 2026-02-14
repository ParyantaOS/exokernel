//! x86_64 architecture-specific initialization.

mod gdt;
mod idt;
pub mod interrupts;

/// Initialize CPU structures (GDT, IDT, PIC) and enable interrupts.
pub fn init() {
    gdt::init();
    idt::init();
    interrupts::init_pic();
    interrupts::enable();
}

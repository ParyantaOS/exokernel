//! Hardware interrupt handling — PIC8259, timer, keyboard.
//!
//! Remaps IRQ 0-15 to interrupt vectors 32-47 to avoid
//! conflicts with CPU exception vectors (0-31).

use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

/// PIC1 starts at vector 32 (right after CPU exceptions 0-31).
pub const PIC_1_OFFSET: u8 = 32;
/// PIC2 starts at vector 40.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Hardware interrupt vector indices.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,        // IRQ0 → vector 32
    Keyboard = PIC_1_OFFSET + 1, // IRQ1 → vector 33
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// Global PIC instance.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Global tick counter (incremented by timer IRQ).
static TICKS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Get the current tick count.
pub fn ticks() -> u64 {
    TICKS.load(core::sync::atomic::Ordering::Relaxed)
}

/// Initialize the 8259 PIC.
pub fn init_pic() {
    unsafe {
        PICS.lock().initialize();
    }
}

/// Enable hardware interrupts.
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

// ─── Interrupt handlers ──────────────────────────────────────────

/// Timer interrupt handler (IRQ0, vector 32).
/// Fires ~18.2 times/sec by default (PIT channel 0).
pub extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    // Decrement scheduler fuel counter
    crate::task::scheduler::timer_tick();

    // Send EOI directly via port I/O to avoid locking PICS mutex
    unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x20).write(0x20);
    }
}

/// Keyboard interrupt handler (IRQ1, vector 33).
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // Use pc-keyboard crate to decode scancodes
    use pc_keyboard::{layouts, DecodedKey, HandleControl, ScancodeSet1};

    lazy_static::lazy_static! {
        static ref KEYBOARD: Mutex<pc_keyboard::Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(pc_keyboard::Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => crate::print!("{}", character),
                DecodedKey::RawKey(key) => crate::print!("{:?}", key),
            }
        }
    }

    // Send EOI directly — PIC1 command port
    unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x20).write(0x20);
    }
}

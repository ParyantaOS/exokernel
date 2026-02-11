//! Memory management for the exokernel.
//!
//! Provides:
//! - Physical frame allocation from bootloader memory map
//! - Kernel heap via linked_list_allocator

pub mod frame_allocator;
pub mod heap;

use bootloader_api::BootInfo;
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::VirtAddr;

use crate::println;

/// Initialize all memory subsystems.
///
/// Must be called after arch::init() and before any heap allocations.
pub fn init(boot_info: &'static BootInfo) {
    let phys_mem_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("bootloader must map physical memory");
    let phys_mem_offset = VirtAddr::new(phys_mem_offset);

    // Set up page table mapper
    let level_4_table = unsafe { active_level_4_table(phys_mem_offset) };
    let mut mapper = unsafe { OffsetPageTable::new(level_4_table, phys_mem_offset) };

    // Initialize frame allocator from bootloader memory map
    let mut frame_allocator =
        unsafe { frame_allocator::BootInfoFrameAllocator::new(&boot_info.memory_regions) };

    let usable_frames = boot_info
        .memory_regions
        .iter()
        .filter(|r| r.kind == bootloader_api::info::MemoryRegionKind::Usable)
        .map(|r| (r.end - r.start) / 4096)
        .sum::<u64>();
    println!("[OK] Frame allocator initialized ({} usable frames, {} MiB)",
        usable_frames,
        usable_frames * 4096 / 1024 / 1024
    );

    // Initialize kernel heap
    heap::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    println!("[OK] Kernel heap initialized ({} KiB)", heap::HEAP_SIZE / 1024);
}

/// Get a mutable reference to the active level 4 page table.
///
/// # Safety
/// - `physical_memory_offset` must be the correct offset that the bootloader
///   used to map all physical memory.
/// - Must only be called once to avoid aliasing `&mut` references.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

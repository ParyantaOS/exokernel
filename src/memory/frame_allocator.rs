//! Physical frame allocator using the bootloader's memory map.
//!
//! This is a simple bump allocator — frames are not freed.
//! Sufficient for kernel heap setup; a more sophisticated allocator
//! (bitmap or buddy) will be added in a future phase.

use bootloader_api::info::{MemoryRegionKind, MemoryRegion};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

/// A frame allocator that yields usable frames from the bootloader memory map.
pub struct BootInfoFrameAllocator {
    memory_regions: &'static [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a new frame allocator from the bootloader memory map.
    ///
    /// # Safety
    /// The caller must guarantee that the memory map is valid and that
    /// all `Usable` regions are truly unused.
    pub unsafe fn new(memory_regions: &'static [MemoryRegion]) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over all usable physical frames.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| {
                let start = r.start;
                let end = r.end;
                let frame_count = (end - start) / 4096;
                (0..frame_count).map(move |i| {
                    let addr = PhysAddr::new(start + i * 4096);
                    PhysFrame::containing_address(addr)
                })
            })
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

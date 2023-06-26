use limine::{LimineMemmapRequest, LimineMemoryMapEntryType};
use x86::current::paging::{PAddr, BASE_PAGE_SIZE};

use super::FrameAllocator;

/// A bump allocator.
///
/// This allocator allocates frames from a contiguous region of memory.
///
/// It consists of a list of bumpers, each of which represents a contiguous
/// region of memory.
///
pub struct BumpAllocator {
    bumpers: [Option<Bumper>; 512],
}

#[derive(Debug, Clone, Copy)]
pub struct Bumper {
    pub heap_start: usize,
    pub heap_end: usize,
    pub next: usize,
}

impl BumpAllocator {
    /// Creates a new `BumpAllocator`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it should only be called once.
    /// unless the previous bump allocator has been dropped.
    ///
    pub unsafe fn new() -> Self {
        static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);

        let entries = MEMMAP_REQUEST
            .get_response()
            .get()
            .unwrap()
            .memmap()
            .iter()
            .filter(|entry| entry.typ == LimineMemoryMapEntryType::Usable);

        let mut bumpers = [None; 512];

        for (i, entry) in entries.enumerate() {
            bumpers[i] = Some(Bumper {
                heap_start: entry.base as usize,
                heap_end: entry.base as usize + entry.len as usize,
                next: entry.base as usize,
            });

            trace!(
                "Bumper {}: {:#x} - {:#x} ({} pages, {} bytes)",
                i,
                entry.base,
                entry.base + entry.len,
                entry.len as usize / BASE_PAGE_SIZE,
                entry.len
            );
        }

        let allocator = Self { bumpers };

        let available_memory = allocator.available_memory();

        trace!(
            "BumpAllocator: {} bytes available ({:.2} MiB, {:.2} GiB, {:.2} TiB, {:.2} pages)",
            available_memory,
            available_memory as f64 / 1024.0 / 1024.0,
            available_memory as f64 / 1024.0 / 1024.0 / 1024.0,
            available_memory as f64 / 1024.0 / 1024.0 / 1024.0 / 1024.0,
            available_memory / BASE_PAGE_SIZE
        );

        allocator
    }

    pub fn available_memory(&self) -> usize {
        self.bumpers
            .iter()
            .filter_map(|bumper| bumper.as_ref())
            .map(|bumper| bumper.heap_end - bumper.heap_start)
            .sum()
    }
}

impl FrameAllocator for Bumper {
    fn alloc(&mut self, count: usize) -> Option<PAddr> {
        let alloc_start = self.next;
        let alloc_end = alloc_start + (count * BASE_PAGE_SIZE);

        if alloc_end > self.heap_end {
            trace!(
                "Bumper: Out of memory! {:#x} > {:#x} ({} pages, {} bytes)",
                alloc_end,
                self.heap_end,
                count,
                count * BASE_PAGE_SIZE
            );
            return None;
        }

        self.next = alloc_end;

        trace!(
            "Bumper: {:#x} - {:#x} ({} pages)",
            alloc_start,
            alloc_end,
            count
        );

        Some(PAddr::from(alloc_start))
    }
}

impl FrameAllocator for BumpAllocator {
    fn alloc(&mut self, count: usize) -> Option<PAddr> {
        self.bumpers
            .iter_mut()
            .filter_map(|bumper| bumper.as_mut())
            .find_map(|bumper| bumper.alloc(count))
    }
}

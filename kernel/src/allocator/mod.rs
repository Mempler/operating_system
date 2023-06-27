use core::alloc::{GlobalAlloc, Layout};

use x86::current::paging::{PAddr, BASE_PAGE_SIZE};

use self::bump::BumpAllocator;

mod bump;

pub trait FrameAllocator {
    /// Allocates a frame of the given size and alignment.
    ///
    /// # Arguments
    /// * `count`: The number of frames to allocate.
    ///
    fn alloc(&mut self, count: usize) -> Option<PAddr>;
}

pub(super) fn init() {
    static mut IS_INITIALIZED: bool = false;

    unsafe {
        if IS_INITIALIZED {
            panic!("Allocator already initialized!");
        }

        IS_INITIALIZED = true;
    }

    let bump = unsafe { BumpAllocator::new() };
    unsafe {
        HEAP.init(bump);
    }

    info!("Initialized heap");
}

#[global_allocator]
pub static mut HEAP: Heap = Heap::new();

pub struct Heap {
    // FIXME: Use a more sophisticated allocator.
    //        a bump allocator is really simple but also huge memory leaks
    //        since we cannot reclaim memory

    // FIXME: given that rust is more optimized for a slab allocator, we should
    //        probably use that instead
    pub allocator: Option<BumpAllocator>,
}

impl Heap {
    pub const fn new() -> Self {
        Heap { allocator: None }
    }

    pub unsafe fn init(&mut self, allocator: BumpAllocator) {
        self.allocator = Some(allocator);
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // we assume that the heap is initialized
        let allocator = HEAP.allocator.as_mut().unwrap();

        // The BumpAllocator can only allocate in frames, so we need to figure out the count of
        // frames we need to allocate.

        let frame_count = (layout.size() + BASE_PAGE_SIZE - 1) / BASE_PAGE_SIZE;
        let frame = allocator.alloc(frame_count).unwrap_or(PAddr::zero());

        frame.0 as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocator does not support dealloc
    }
}

pub unsafe fn allocate_pages(count: usize) -> PAddr {
    HEAP.allocator.as_mut().unwrap().alloc(count).unwrap()
}

pub unsafe fn deallocate_pages(addr: PAddr, count: usize) {
    unimplemented!("deallocate_pages")
}

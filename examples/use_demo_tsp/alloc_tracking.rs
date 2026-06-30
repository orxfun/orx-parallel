use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct TrackingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;

static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static DEALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static REALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_OLD_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_NEW_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        REALLOC_OLD_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        REALLOC_NEW_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct AllocationStats {
    pub alloc_calls: u64,
    pub dealloc_calls: u64,
    pub realloc_calls: u64,
    pub alloc_bytes: u64,
    pub dealloc_bytes: u64,
    pub realloc_old_bytes: u64,
    pub realloc_new_bytes: u64,
}

impl AllocationStats {
    pub fn reset() {
        ALLOC_CALLS.store(0, Ordering::Relaxed);
        DEALLOC_CALLS.store(0, Ordering::Relaxed);
        REALLOC_CALLS.store(0, Ordering::Relaxed);
        ALLOC_BYTES.store(0, Ordering::Relaxed);
        DEALLOC_BYTES.store(0, Ordering::Relaxed);
        REALLOC_OLD_BYTES.store(0, Ordering::Relaxed);
        REALLOC_NEW_BYTES.store(0, Ordering::Relaxed);
    }

    pub fn read() -> Self {
        Self {
            alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
            dealloc_calls: DEALLOC_CALLS.load(Ordering::Relaxed),
            realloc_calls: REALLOC_CALLS.load(Ordering::Relaxed),
            alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
            dealloc_bytes: DEALLOC_BYTES.load(Ordering::Relaxed),
            realloc_old_bytes: REALLOC_OLD_BYTES.load(Ordering::Relaxed),
            realloc_new_bytes: REALLOC_NEW_BYTES.load(Ordering::Relaxed),
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            alloc_calls: self.alloc_calls + other.alloc_calls,
            dealloc_calls: self.dealloc_calls + other.dealloc_calls,
            realloc_calls: self.realloc_calls + other.realloc_calls,
            alloc_bytes: self.alloc_bytes + other.alloc_bytes,
            dealloc_bytes: self.dealloc_bytes + other.dealloc_bytes,
            realloc_old_bytes: self.realloc_old_bytes + other.realloc_old_bytes,
            realloc_new_bytes: self.realloc_new_bytes + other.realloc_new_bytes,
        }
    }

    pub fn div(self, n: u64) -> Self {
        Self {
            alloc_calls: self.alloc_calls / n,
            dealloc_calls: self.dealloc_calls / n,
            realloc_calls: self.realloc_calls / n,
            alloc_bytes: self.alloc_bytes / n,
            dealloc_bytes: self.dealloc_bytes / n,
            realloc_old_bytes: self.realloc_old_bytes / n,
            realloc_new_bytes: self.realloc_new_bytes / n,
        }
    }

    pub fn gross_allocated_bytes(&self) -> u64 {
        self.alloc_bytes + self.realloc_new_bytes
    }

    pub fn gross_released_bytes(&self) -> u64 {
        self.dealloc_bytes + self.realloc_old_bytes
    }
}

pub mod x86_64;

pub use x86_64::hcf;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddress(u64);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(u64);

pub trait MemoryMapper {
    unsafe fn new() -> Self;
    unsafe fn from_active() -> Self;

    unsafe fn map(&mut self, phys: PhysicalAddress, virt: VirtualAddress, flags: u64);
    unsafe fn unmap(&mut self, virt: VirtualAddress);

    unsafe fn submit(&mut self);
}

pub type HalMemoryMapper = x86_64::mmu::X64MemoryMapper;

use core::arch::asm;

use alloc::{boxed::Box, vec::Vec};
use limine::{LimineSmpInfo, LimineSmpRequest};
use spin::Mutex;

use crate::allocator;

pub mod acpi;
pub mod gdt;
pub mod idt;
pub mod mmu;

unsafe fn common_startup() {
    idt::disable();

    gdt::init();
    idt::init();

    idt::enable();
}

#[no_mangle]
extern "C" fn bsp_start() -> ! {
    klogger::init("trace", 0x3F8).unwrap();

    unsafe {
        common_startup();
    }

    allocator::init();
    mmu::init();

    info!("CPU - 0 (BSP) started");

    crate::bsp_main();
}

#[no_mangle]
extern "C" fn ap_start(info: *const LimineSmpInfo) -> ! {
    let info = unsafe { &*info };

    info!("CPU - {} (AP) started", info.processor_id);

    crate::ap_main();
}

pub fn hcf() -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}

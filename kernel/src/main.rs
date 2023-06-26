//!
//! *TODO: Add docs here*
//!

#![deny(missing_docs)]
#![no_std]
#![no_main]

use core::arch::asm;

mod allocator;
mod arch;

#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate klogger;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    klogger::init("trace", 0x3F8).unwrap();

    arch::x86_64::gdt::init();

    allocator::init();

    hcf();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hcf();
}

fn hcf() -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}

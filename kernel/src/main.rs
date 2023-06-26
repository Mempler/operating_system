//!
//! *TODO: Add docs here*
//!

#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![allow(unused)]

use core::arch::asm;

mod allocator;
mod arch;

#[macro_use]
extern crate log;

#[macro_use]
extern crate klogger;

#[macro_use]
extern crate alloc;

extern crate static_assertions as sa;

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    klogger::init("trace", 0x3F8).unwrap();

    arch::x86_64::gdt::init();
    arch::x86_64::idt::init();

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

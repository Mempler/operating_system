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

fn pre_init() {
    klogger::init("trace", 0x3F8).unwrap();
}

fn main() -> ! {
    allocator::init();

    arch::hcf();
}

#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    arch::hcf();
}

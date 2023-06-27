//!
//! *TODO: Add docs here*
//!

#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
#![allow(unused)]

use core::{arch::asm, panic::PanicInfo};

mod allocator;
mod arch;

#[macro_use]
extern crate log;

#[macro_use]
extern crate klogger;

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

extern crate static_assertions as sa;

fn bsp_main() -> ! {
    arch::hcf();
}

fn ap_main() -> ! {
    arch::hcf();
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    error!("Kernel panic: {}", info);
    arch::hcf();
}

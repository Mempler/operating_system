use core::arch::asm;

use limine::{LimineSmpInfo, LimineSmpRequest};

pub mod gdt;
pub mod idt;

static SMP_REQUEST: LimineSmpRequest = LimineSmpRequest::new(0);

unsafe fn common_startup() {
    idt::disable();

    gdt::init();
    idt::init();

    idt::enable();
}

#[no_mangle]
extern "C" fn bsp_start() -> ! {
    crate::pre_init();

    unsafe {
        common_startup();
    }

    let smp_response = SMP_REQUEST.get_response().get_mut().unwrap();

    for cpu in smp_response.cpus() {
        cpu.goto_address = ap_start;
    }

    info!("CPU - 0 (BSP) started");

    crate::bsp_main();
}

#[no_mangle]
extern "C" fn ap_start(info: *const LimineSmpInfo) -> ! {
    let info = unsafe { &*info };

    unsafe {
        common_startup();
    }

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

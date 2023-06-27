use alloc::boxed::Box;
use x86::{
    controlregs::{cr3, cr3_write},
    current::paging::*,
};

use crate::{
    allocator::{allocate_pages, FrameAllocator},
    arch::MemoryMapper,
};

pub fn init() {
    unsafe {
        // For now, we'll just use limine's memory map
        let mut mapper = X64MemoryMapper::from_active();

        mapper.submit();
    }
}

#[repr(transparent)]
pub struct X64MemoryMapper {
    pml4: &'static mut PML4,
}

impl X64MemoryMapper {
    unsafe fn get_pdpt(&mut self, pml4_idx: usize) -> Option<*mut PDPT> {
        let pml4 = &mut self.pml4[pml4_idx];

        if pml4.is_present() {
            Some(pml4.address().0 as *mut PDPT)
        } else {
            None
        }
    }

    unsafe fn get_pd(&mut self, pml4_idx: usize, pdp_idx: usize) -> Option<*mut PD> {
        let pdpt = self.get_pdpt(pml4_idx)?;

        let pdpt = &mut *pdpt;
        let pdp = &mut pdpt[pdp_idx];

        if pdp.is_present() {
            Some(pdp.address().0 as *mut PD)
        } else {
            None
        }
    }

    unsafe fn get_pt(&mut self, pml4_idx: usize, pdp_idx: usize, pd_idx: usize) -> Option<*mut PT> {
        let pd = self.get_pd(pml4_idx, pdp_idx)?;

        let pd = &mut *pd;
        let pt = &mut pd[pd_idx];

        if pt.is_present() {
            Some(pt.address().0 as *mut PT)
        } else {
            None
        }
    }

    unsafe fn alloc_pdpt(&mut self, pml4_idx: usize, flags: u64) -> *mut PDPT {
        let frame = allocate_pages(1).as_u64();
        let pdpt = &mut self.pml4[pml4_idx];

        pdpt.0 = frame | flags;

        trace!("Allocated PDPT at {:#x}", frame);

        frame as *mut PDPT
    }

    unsafe fn alloc_pd(&mut self, pml4_idx: usize, pdp_idx: usize, flags: u64) -> *mut PD {
        let frame = allocate_pages(1).as_u64();
        let pdpt = self.get_pdpt(pml4_idx).unwrap();

        let pdpt = &mut *pdpt;
        let pdp = &mut pdpt[pdp_idx];

        pdp.0 = frame | flags;

        trace!("Allocated PD at {:#x}", frame);

        frame as *mut PD
    }

    unsafe fn alloc_pt(
        &mut self,
        pml4_idx: usize,
        pdp_idx: usize,
        pd_idx: usize,
        flags: u64,
    ) -> *mut PT {
        let frame = allocate_pages(1).as_u64();
        let pd = self.get_pd(pml4_idx, pdp_idx).unwrap();

        let pd = &mut *pd;
        let pt = &mut pd[pd_idx];

        pt.0 = frame | flags;

        trace!("Allocated PT at {:#x}", frame);

        frame as *mut PT
    }

    // Debugging
    // -------------------------------------------------------------------------

    pub fn dump_to_trace(&self) {
        trace!("PML4:");

        for (pml_idx, pml4_entry) in self.pml4.iter().enumerate() {
            if !pml4_entry.is_present() {
                continue;
            }

            let pdpt = unsafe { pml4_entry.address().0 as *const PDPT };

            trace!("  PDPT {:#x}:", pml_idx);
            for (pdp_idx, pdpt_entry) in unsafe { &*pdpt }.iter().enumerate() {
                if !pdpt_entry.is_present() {
                    continue;
                }

                let pd = unsafe { pdpt_entry.address().0 as *const PD };

                trace!("    PD {:#x}:", pdp_idx);
                for (pd_idx, pd_entry) in unsafe { &*pd }.iter().enumerate() {
                    if !pd_entry.is_present() {
                        continue;
                    }

                    let pt = unsafe { pd_entry.address().0 as *const PT };

                    trace!("      PT {:#x}:", pd_idx);
                    for (pt_idx, pt_entry) in unsafe { &*pt }.iter().enumerate() {
                        if !pt_entry.is_present() {
                            continue;
                        }

                        let phys = pt_entry.address().0;
                        let virt = VAddr::from_u64(
                            (pml_idx as u64) << 39
                                | (pdp_idx as u64) << 30
                                | (pd_idx as u64) << 21
                                | (pt_idx as u64) << 12,
                        );

                        trace!("        {:#x} -> {:#x}", virt, phys);
                    }
                }
            }
        }
    }
}

impl MemoryMapper for X64MemoryMapper {
    unsafe fn new() -> Self {
        let pml4 = allocate_pages(1);

        X64MemoryMapper {
            pml4: &mut *(pml4.as_u64() as *mut PML4),
        }
    }

    unsafe fn from_active() -> Self {
        let pml4 = cr3() as *mut PML4;
        X64MemoryMapper { pml4: &mut *pml4 }
    }

    unsafe fn map(
        &mut self,
        phys: crate::arch::PhysicalAddress,
        virt: crate::arch::VirtualAddress,
        flags: u64,
    ) {
        let pml_idx: usize = (virt.0 >> 39 & 0o777u64) as _; // 9 bits
        let pdp_idx: usize = (virt.0 >> 30 & 0o777u64) as _; // 9 bits
        let pd_idx: usize = (virt.0 >> 21 & 0o777u64) as _; // 9 bits
        let pt_idx: usize = (virt.0 >> 12 & 0o777u64) as _; // 9 bits

        trace!(
            "Mapping {:#x} -> {:#x} (pml_idx: {:#x}, pdp_idx: {:#x}, pd_idx: {:#x}, pt_idx: {:#x})",
            virt.0,
            phys.0,
            pml_idx,
            pdp_idx,
            pd_idx,
            pt_idx
        );

        let pt = match self.get_pt(pml_idx, pdp_idx, pd_idx) {
            Some(pt) => pt,
            None => {
                let pt = self.alloc_pt(pml_idx, pdp_idx, pd_idx, flags);
                let pt = &mut *pt;

                for entry in pt.iter_mut() {
                    entry.0 = 0;
                }

                pt
            }
        };

        let pt = &mut *pt;
        let pt_entry = &mut pt[pt_idx];

        pt_entry.0 = phys.0 | flags;
    }

    unsafe fn unmap(&mut self, virt: crate::arch::VirtualAddress) {
        let pml_idx: usize = (virt.0 >> 39 & 0o777u64) as _; // 9 bits
        let pdp_idx: usize = (virt.0 >> 30 & 0o777u64) as _; // 9 bits
        let pd_idx: usize = (virt.0 >> 21 & 0o777u64) as _; // 9 bits
        let pt_idx: usize = (virt.0 >> 12 & 0o777u64) as _; // 9 bits

        trace!(
            "Unmapping {:#x} (pml_idx: {:#x}, pdp_idx: {:#x}, pd_idx: {:#x}, pt_idx: {:#x})",
            virt.0,
            pml_idx,
            pdp_idx,
            pd_idx,
            pt_idx
        );

        let pt = match self.get_pt(pml_idx, pdp_idx, pd_idx) {
            Some(pt) => pt,
            None => return,
        };

        let pt = &mut *pt;
        let pt_entry = &mut pt[pt_idx];

        pt_entry.0 = 0;
    }

    unsafe fn submit(&mut self) {
        let tst = self.pml4 as *const PML4;
        trace!("Submitting PML4 {:#x}", tst as u64);
        let tst2 = cr3();
        trace!("Submitting PML4 {:#x}", tst2);

        let pml4 = self.pml4 as *mut PML4;
        cr3_write(pml4 as u64);
    }
}

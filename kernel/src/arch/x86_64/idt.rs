use x86::{
    current::segmentation::Descriptor64,
    dtables::{lidt, DescriptorTablePointer},
    irq::{
        self, InterruptDescription, ALIGNMENT_CHECK_VECTOR, DOUBLE_FAULT_VECTOR, EXCEPTIONS,
        GENERAL_PROTECTION_FAULT_VECTOR, INVALID_TSS_VECTOR, PAGE_FAULT_VECTOR,
        SEGMENT_NOT_PRESENT_VECTOR, STACK_SEGEMENT_FAULT_VECTOR,
    },
    segmentation::{BuildDescriptor, DescriptorBuilder, GateDescriptorBuilder, SegmentSelector},
    Ring,
};

#[repr(transparent)]
pub struct InterruptDescriptorTable([Descriptor64; 256]);
sa::const_assert_eq!(core::mem::size_of::<InterruptDescriptorTable>(), 4096);

impl InterruptDescriptorTable {
    pub const fn new() -> Self {
        Self([Descriptor64::NULL; 256])
    }

    /// Loads the interrupt description table.
    ///
    pub fn load(&'static self) {
        let idt_ptr = DescriptorTablePointer::new(&self.0);
        unsafe {
            lidt(&idt_ptr);
        }
    }

    /// Returns the interrupt description for the given index.
    ///
    /// This will only work for exceptions within the range of 0-31.
    ///
    pub fn get_description(&self, index: usize) -> Option<&'static InterruptDescription> {
        if index >= 32 {
            return None;
        }

        Some(&EXCEPTIONS[index])
    }

    /// Sets the interrupt handler for the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the interrupt to set the handler for.
    /// * `segment` - The code segment to use for the interrupt handler.
    /// * `ist` - The interrupt stack table index to use for the interrupt handler.
    /// * `dpl` - The privilege level to use for the interrupt handler.
    /// * `handler` - The interrupt handler function.
    ///
    pub fn set_interrupt(
        &mut self,
        index: u8,
        segment: SegmentSelector,
        ist: u8,
        dpl: Ring,
        handler: unsafe extern "x86-interrupt" fn(stack_frame: ExceptionStackFrame),
    ) {
        // Some interrupts can contain error codes, we cannot use this function for them
        assert!(
            index != DOUBLE_FAULT_VECTOR
                && index != INVALID_TSS_VECTOR
                && index != SEGMENT_NOT_PRESENT_VECTOR
                && index != STACK_SEGEMENT_FAULT_VECTOR
                && index != GENERAL_PROTECTION_FAULT_VECTOR
                && index != PAGE_FAULT_VECTOR
                && index != ALIGNMENT_CHECK_VECTOR
        );

        // Some interrupts are reserved and cannot be used
        assert!(index != 15 && !(index >= 21 && index <= 31));

        self.0[index as usize] = DescriptorBuilder::interrupt_descriptor(segment, handler as u64)
            .present()
            .ist(ist)
            .dpl(dpl)
            .finish();
    }

    /// Sets the interrupt handler for the given index but have a error code
    ///
    /// # Arguments
    /// * `index` - The index of the interrupt to set the handler for.
    /// * `segment` - The code segment to use for the interrupt handler.
    /// * `ist` - The interrupt stack table index to use for the interrupt handler.
    /// * `dpl` - The privilege level to use for the interrupt handler.
    /// * `handler` - The interrupt handler function.
    ///
    pub fn set_interrupt_with_error_code(
        &mut self,
        index: u8,
        segment: SegmentSelector,
        ist: u8,
        dpl: Ring,
        handler: unsafe extern "x86-interrupt" fn(
            stack_frame: ExceptionStackFrame,
            error_code: u32,
        ),
    ) {
        // We have to make sure that the interrupt we are setting up is one that has an error code
        assert!(
            index == DOUBLE_FAULT_VECTOR
                && index == INVALID_TSS_VECTOR
                && index == SEGMENT_NOT_PRESENT_VECTOR
                && index == STACK_SEGEMENT_FAULT_VECTOR
                && index == GENERAL_PROTECTION_FAULT_VECTOR
                && index == PAGE_FAULT_VECTOR
                && index == ALIGNMENT_CHECK_VECTOR
                && (index >= 31)
        );

        self.0[index as usize] = DescriptorBuilder::interrupt_descriptor(segment, handler as u64)
            .present()
            .ist(ist)
            .dpl(dpl)
            .finish();
    }

    /// Sets the trap handler for the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the trap to set the handler for.
    /// * `segment` - The code segment to use for the trap handler.
    /// * `ist` - The interrupt stack table index to use for the trap handler.
    /// * `dpl` - The privilege level to use for the trap handler.
    /// * `handler` - The trap handler function.
    ///
    pub fn set_trap(
        &mut self,
        index: u8,
        segment: SegmentSelector,
        ist: u8,
        dpl: Ring,
        handler: unsafe extern "x86-interrupt" fn(stack_frame: ExceptionStackFrame),
    ) {
        // Some interrupts can contain error codes, we cannot use this function for them
        assert!(
            index != DOUBLE_FAULT_VECTOR
                && index != INVALID_TSS_VECTOR
                && index != SEGMENT_NOT_PRESENT_VECTOR
                && index != STACK_SEGEMENT_FAULT_VECTOR
                && index != GENERAL_PROTECTION_FAULT_VECTOR
                && index != PAGE_FAULT_VECTOR
                && index != ALIGNMENT_CHECK_VECTOR
        );

        // Some interrupts are reserved and cannot be used
        assert!(index != 15 && !(index >= 21 && index <= 31));

        self.0[index as usize] = DescriptorBuilder::trap_gate_descriptor(segment, handler as u64)
            .present()
            .ist(ist)
            .dpl(dpl)
            .finish();
    }

    /// Sets the trap handler for the given index but have a error code
    ///
    /// # Arguments
    /// * `index` - The index of the trap to set the handler for.
    /// * `segment` - The code segment to use for the trap handler.
    /// * `ist` - The interrupt stack table index to use for the trap handler.
    /// * `dpl` - The privilege level to use for the trap handler.
    /// * `handler` - The trap handler function.
    ///
    pub fn set_trap_with_error_code(
        &mut self,
        index: u8,
        segment: SegmentSelector,
        ist: u8,
        dpl: Ring,
        handler: unsafe extern "x86-interrupt" fn(
            stack_frame: ExceptionStackFrame,
            error_code: u32,
        ),
    ) {
        // We have to make sure that the interrupt we are setting up is one that has an error code
        assert!(
            index == DOUBLE_FAULT_VECTOR
                && index == INVALID_TSS_VECTOR
                && index == SEGMENT_NOT_PRESENT_VECTOR
                && index == STACK_SEGEMENT_FAULT_VECTOR
                && index == GENERAL_PROTECTION_FAULT_VECTOR
                && index == PAGE_FAULT_VECTOR
                && index == ALIGNMENT_CHECK_VECTOR
                && (index >= 31)
        );

        self.0[index as usize] = DescriptorBuilder::trap_gate_descriptor(segment, handler as u64)
            .present()
            .ist(ist)
            .dpl(dpl)
            .finish();
    }

    /// Disables the interrupt handler for the given index.
    ///
    /// # Arguments
    /// * `index` - The index of the interrupt to disable.
    ///
    pub fn unset(&mut self, index: u8) {
        self.0[index as usize].set_ist(index)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub unsafe fn init() {
    disable();

    IDT.load();

    enable();
}

pub fn enable() {
    unsafe {
        irq::enable();
    }
}

pub fn disable() {
    unsafe {
        irq::disable();
    }
}

use x86::{
    dtables::{lgdt, DescriptorTablePointer},
    segmentation::{
        load_cs, load_ds, load_es, load_fs, load_gs, load_ss, BuildDescriptor, CodeSegmentType,
        DataSegmentType, Descriptor, DescriptorBuilder, SegmentDescriptorBuilder, SegmentSelector,
    },
    Ring,
};

use super::idt;

static mut GDT: [Descriptor; 8] = [Descriptor::NULL; 8];

pub unsafe fn init() {
    let code_kernel = DescriptorBuilder::code_descriptor(0, 0xFFFFF, CodeSegmentType::ExecuteRead)
        .present()
        .dpl(Ring::Ring0)
        .limit_granularity_4kb()
        .db()
        .l()
        .finish();

    let data_kernel = DescriptorBuilder::data_descriptor(0, 0xFFFFF, DataSegmentType::ReadWrite)
        .present()
        .dpl(Ring::Ring0)
        .limit_granularity_4kb()
        .db()
        .l()
        .finish();

    let code_user = DescriptorBuilder::code_descriptor(0, 0xFFFFF, CodeSegmentType::ExecuteRead)
        .present()
        .limit_granularity_4kb()
        .db()
        .dpl(Ring::Ring3)
        .l()
        .finish();

    let data_user = DescriptorBuilder::data_descriptor(0, 0xFFFFF, DataSegmentType::ReadWrite)
        .present()
        .limit_granularity_4kb()
        .db()
        .dpl(Ring::Ring3)
        .l()
        .finish();

    GDT[1] = code_kernel;
    GDT[2] = data_kernel;
    GDT[3] = code_user;
    GDT[4] = data_user;

    idt::disable();

    let gdt_ptr = DescriptorTablePointer::new(&GDT);
    lgdt(&gdt_ptr);

    let code_segment = SegmentSelector::new(1, Ring::Ring0);
    let data_segment = SegmentSelector::new(2, Ring::Ring0);

    load_ss(data_segment);
    load_ds(data_segment);
    load_es(data_segment);
    load_fs(data_segment);
    load_gs(data_segment);

    load_cs(code_segment);

    idt::enable();
}

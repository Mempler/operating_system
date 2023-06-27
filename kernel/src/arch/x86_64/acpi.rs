use acpi::AcpiHandler;
use limine::LimineRsdpRequest;

static mut ACPI_RSDP_REQUEST: LimineRsdpRequest = LimineRsdpRequest::new(0);

type AcpiTables = acpi::AcpiTables<AcpiMapper>;

pub unsafe fn init() {
    let acpi_tables = {
        let addr = ACPI_RSDP_REQUEST
            .get_response()
            .get_mut()
            .unwrap()
            .address
            .as_ptr()
            .unwrap();

        AcpiTables::from_rsdp(AcpiMapper, addr as usize)
    };
}

#[derive(Debug, Clone, Copy)]
struct AcpiMapper;

impl AcpiHandler for AcpiMapper {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        todo!()
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        todo!()
    }
}

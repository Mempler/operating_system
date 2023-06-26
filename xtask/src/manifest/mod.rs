mod disk;
mod qemu;

pub use disk::*;
pub use qemu::*;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Manifest {
    pub qemu: ManifestQemu,
    pub disk: ManifestDisk,
}

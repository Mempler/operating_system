use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
    path::{Path, PathBuf},
};

use byte_unit::Byte;
use fscommon::StreamSlice;
use gpt::{mbr::ProtectiveMBR, partition::Partition, partition_types::Type, GptConfig};
use tokio::io::AsyncReadExt;
use url::Url;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::resolve::resolve;

#[derive(Debug, thiserror::Error)]
pub enum ManifestDiskError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Resolve(#[from] crate::resolve::ResolveError),

    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),

    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
}

type Result<T, E = ManifestDiskError> = std::result::Result<T, E>;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestDisk {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub table: ManifestDiskTable,
    pub size: Byte,
    pub partitions: Vec<ManifestDiskPartition>,
}

impl ManifestDisk {
    pub async fn build(&self, int_dir: &Path) -> Result<PathBuf> {
        let disk_dir = int_dir.join(self.id.to_string());

        tokio::fs::create_dir_all(&disk_dir).await?;

        let disk_path = int_dir.join(format!("{}.img", self.id));

        trace!("Creating disk image at {}", disk_path.display());
        let mut disk_file = Box::new(
            std::fs::File::options()
                .create(true)
                .write(true)
                .read(true)
                .truncate(true)
                .open(&disk_path)?,
        );
        disk_file.set_len(self.size.get_bytes() as u64)?;

        trace!("Writing protective MBR");
        ProtectiveMBR::with_lb_size(
            u32::try_from((self.size.get_bytes() / 512) - 1).unwrap_or(0xFF_FF_FF_FF),
        )
        .overwrite_lba0(&mut disk_file)?;

        let mut disk = GptConfig::new()
            .initialized(false)
            .writable(true)
            .logical_block_size(gpt::disk::LogicalBlockSize::Lb512)
            .create_from_device(disk_file, Some(self.id))?;

        trace!("Initializing empty GPT");
        disk.update_partitions(BTreeMap::<u32, Partition>::new())?;

        trace!("Modifying GPT with partitions");

        for part in &self.partitions {
            trace!(
                "Creating partition {} <{}> :: {:#X}",
                part.name,
                part.id,
                part.size.get_bytes()
            );

            let part_dir = disk_dir.join(part.id.to_string());
            tokio::fs::create_dir_all(&part_dir).await?;

            // This leak is required due to the need of a static lifetime
            let type_guid = Box::leak(part.part_type.to_string().into_boxed_str());

            let id = disk.add_partition(
                &part.name,
                part.size.get_bytes() as u64,
                Type {
                    guid: type_guid,
                    os: gpt::partition_types::OperatingSystem::None,
                },
                0, // TODO: implement flags
                None,
            )?;

            let mut partitions = disk.partitions().clone();
            let partition = partitions.get_mut(&id).unwrap();
            partition.part_guid = part.id;

            disk.update_partitions(partitions)?;
        }

        trace!("Writing GPT to disk");

        let partitions = disk.partitions().clone();
        let mut disk_file = disk.write()?;

        trace!("Resolving files");

        for part in &self.partitions {
            let part_dir = disk_dir.join(part.id.to_string());

            for (dst, src) in &part.files {
                let dst = part_dir.join(dst.strip_prefix("/")?);

                resolve(src, Some(&dst)).await?;
            }
        }

        trace!("Writing filesystems to disk");

        for (_id, gpt_part) in partitions {
            let mut buf = StreamSlice::new(
                &mut disk_file,
                gpt_part.first_lba * 512,
                gpt_part.last_lba * 512,
            )?;

            let part = self
                .partitions
                .iter()
                .find(|p| p.id == gpt_part.part_guid)
                .unwrap();

            let part_dir = disk_dir.join(part.id.to_string());

            trace!(
                "start_sector: {:#x} end_sector: {:#x} size: {:#x}",
                gpt_part.first_lba,
                gpt_part.last_lba,
                gpt_part.size()?,
            );

            let mut files = vec![];
            for entry in WalkDir::new(&part_dir) {
                let entry = entry?;

                if entry.file_type().is_file() || entry.file_type().is_symlink() {
                    files.push(entry.path().to_path_buf());
                }
            }

            match part.filesystem {
                ManifestDiskFilesystem::FAT32 => {
                    trace!("Formatting FAT32 filesystem");
                    fatfs::format_volume(
                        &mut buf,
                        fatfs::FormatVolumeOptions::new()
                            .bytes_per_sector(512)
                            .total_sectors(gpt_part.size()? as u32),
                    )?;

                    let fs = fatfs::FileSystem::new(buf, fatfs::FsOptions::new())?;
                    let root = fs.root_dir();

                    for file_path in files {
                        let mut file = tokio::fs::File::open(&file_path).await?;

                        let dst_file_path = file_path.strip_prefix(&part_dir).unwrap();

                        let mut dir = root.clone();
                        for component in dst_file_path.parent().unwrap().components() {
                            dir = dir.create_dir(component.as_os_str().to_str().unwrap())?;
                        }

                        let mut dst_file =
                            dir.create_file(dst_file_path.file_name().unwrap().to_str().unwrap())?;

                        let mut file_data = vec![0; 0x500];
                        loop {
                            let read = file.read(&mut file_data).await?;

                            if read == 0 {
                                break;
                            }

                            dst_file.write_all(&file_data[..read])?;
                        }
                    }
                }
            }
        }

        Ok(disk_path)
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub enum ManifestDiskTable {
    #[default]
    #[serde(rename = "gpt")]
    GPT,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ManifestDiskFilesystem {
    #[serde(rename = "fat32")]
    FAT32,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub enum ManifestDiskFilesystemFlags {
    #[serde(rename = "boot")]
    Boot,

    #[serde(rename = "esp")]
    ESP,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ManifestDiskPartition {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    #[serde(rename = "type")]
    pub part_type: Uuid,
    pub name: String,
    pub flags: Vec<ManifestDiskFilesystemFlags>,
    pub size: Byte,
    pub filesystem: ManifestDiskFilesystem,
    pub files: HashMap<PathBuf, Url>,
}

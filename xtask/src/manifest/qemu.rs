use byte_unit::Byte;
use std::fmt::Display;
use url::Url;

use crate::resolve::resolve;

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestQemu {
    machine: Option<ManifestQemuMachine>,
    cpu: Option<ManifestQemuCpu>,
    memory: Option<ManifestQemuMemory>,
    drives: Option<Vec<ManifestQemuDrive>>,
    serial: Option<String>,
    extra_args: Option<Vec<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestQemuError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Resolve(#[from] crate::resolve::ResolveError),
}

type Result<T, E = ManifestQemuError> = std::result::Result<T, E>;

impl ManifestQemu {
    pub async fn exec(&self) -> Result<()> {
        let mut args = vec![];

        self.machine.as_ref().map(|m| m.to_args(&mut args));
        self.cpu.as_ref().map(|c| c.to_args(&mut args));
        self.memory.as_ref().map(|m| m.to_args(&mut args));

        if let Some(drives) = &self.drives {
            for drive in drives {
                drive.to_args(&mut args).await?;
            }
        }

        if let Some(serial) = &self.serial {
            args.push("-serial".to_string());
            args.push(serial.to_string());
        }

        if let Some(extra_args) = &self.extra_args {
            args.extend(extra_args.iter().cloned());
        }

        let final_command = format!("qemu-system-x86_64 {:?}", args);

        trace!("qemu-system-x86_64 {}", final_command);

        tokio::process::Command::new("qemu-system-x86_64")
            .args(args)
            .status()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestQemuMachine {
    #[serde(rename = "type")]
    kind: ManifestQemuMachineType,
    accel: Option<Vec<ManifestQemuMachineAccel>>,
    vmport: Option<bool>,
    dump_guest_core: Option<bool>,
    mem_merge: Option<bool>,
    aes_key_wrap: Option<bool>,
    dae_key_wrap: Option<bool>,
    nvdimm: Option<bool>,
    memory_encryption: Option<String>,
    hmat: Option<bool>,
}

impl ManifestQemuMachine {
    fn to_args(&self, args: &mut Vec<String>) {
        let mut string = String::new();
        let mut accel_string = String::new();

        string.push_str(&format!("type={}", self.kind));
        if let Some(accel) = &self.accel {
            if !accel.is_empty() {
                string.push_str(",accel=");

                for (i, accel) in accel.iter().enumerate() {
                    if i > 0 {
                        string.push_str(":");
                    }

                    string.push_str(&accel.name);

                    if let Some(igd_passthru) = accel.igd_passthru {
                        accel_string.push_str(&format!(
                            ",igd-passthru={}",
                            if igd_passthru { "on" } else { "off" }
                        ));
                    }

                    if let Some(kernel_irqchip) = &accel.kernel_irqchip {
                        accel_string.push_str(&format!(",kernel-irqchip={}", kernel_irqchip));
                    }

                    if let Some(kvm_shadow_mem) = &accel.kvm_shadow_mem {
                        accel_string
                            .push_str(&format!(",kvm-shadow-mem={}", kvm_shadow_mem.get_bytes()));
                    }

                    if let Some(tb_size) = &accel.tb_size {
                        accel_string.push_str(&format!(
                            ",tb-size={}",
                            tb_size.get_bytes() * 1024 * 1024 // In MiB
                        ));
                    }

                    if let Some(thread) = &accel.thread {
                        accel_string.push_str(&format!(",thread={}", thread));
                    }
                }
            }
        }

        if let Some(vmport) = self.vmport {
            string.push_str(&format!(",vmport={}", if vmport { "on" } else { "off" }));
        }

        if let Some(dump_guest_core) = self.dump_guest_core {
            string.push_str(&format!(
                ",dump-guest-core={}",
                if dump_guest_core { "on" } else { "off" }
            ));
        }

        if let Some(mem_merge) = self.mem_merge {
            string.push_str(&format!(
                ",mem-merge={}",
                if mem_merge { "on" } else { "off" }
            ));
        }

        if let Some(aes_key_wrap) = self.aes_key_wrap {
            string.push_str(&format!(
                ",aes-key-wrap={}",
                if aes_key_wrap { "on" } else { "off" }
            ));
        }

        if let Some(dae_key_wrap) = self.dae_key_wrap {
            string.push_str(&format!(
                ",dae-key-wrap={}",
                if dae_key_wrap { "on" } else { "off" }
            ));
        }

        if let Some(nvdimm) = self.nvdimm {
            string.push_str(&format!(",nvdimm={}", if nvdimm { "on" } else { "off" }));
        }

        if let Some(memory_encryption) = &self.memory_encryption {
            string.push_str(&format!(",memory-encryption={}", memory_encryption));
        }

        if let Some(hmat) = self.hmat {
            string.push_str(&format!(",hmat={}", if hmat { "on" } else { "off" }));
        }

        if !string.is_empty() {
            args.push("-machine".to_string());
            args.push(string);
        }

        if !accel_string.is_empty() {
            args.push("-accel".to_string());
            args.push(accel_string);
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub enum ManifestQemuMachineType {
    #[default]
    #[serde(rename = "q35")]
    Q35,

    #[serde(rename = "pc")]
    PC,
}

impl Display for ManifestQemuMachineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Q35 => write!(f, "q35"),
            Self::PC => write!(f, "pc"),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestQemuMachineAccel {
    name: String,
    igd_passthru: Option<bool>,
    kernel_irqchip: Option<ManifestQemuMachineKernelIRQChip>,
    kvm_shadow_mem: Option<Byte>,
    tb_size: Option<Byte>,
    thread: Option<ManifestQemuMachineAccelThread>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub enum ManifestQemuMachineKernelIRQChip {
    #[default]
    #[serde(rename = "on")]
    On,

    #[serde(rename = "off")]
    Off,

    #[serde(rename = "split")]
    Split,
}

impl Display for ManifestQemuMachineKernelIRQChip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
            Self::Split => write!(f, "split"),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub enum ManifestQemuMachineAccelThread {
    #[serde(rename = "single")]
    Single,

    #[default]
    #[serde(rename = "multi")]
    Multi,
}

impl Display for ManifestQemuMachineAccelThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Single => write!(f, "single"),
            Self::Multi => write!(f, "multi"),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestQemuCpu {
    #[serde(rename = "type")]
    kind: String,
    cpus: Option<usize>,
    cores: Option<usize>,
    threads: Option<usize>,
    dies: Option<usize>,
    sockets: Option<usize>,
    maxcpus: Option<usize>,
}

impl ManifestQemuCpu {
    fn to_args(&self, args: &mut Vec<String>) {
        let mut string = String::new();
        let mut smp_string = String::new();

        string.push_str(&format!("{}", self.kind));

        if let Some(cpus) = self.cpus {
            smp_string.push_str(&format!("cpus={}", cpus));
        }

        if let Some(cores) = self.cores {
            smp_string.push_str(&format!(",cores={}", cores));
        }

        if let Some(threads) = self.threads {
            smp_string.push_str(&format!(",threads={}", threads));
        }

        if let Some(dies) = self.dies {
            smp_string.push_str(&format!(",dies={}", dies));
        }

        if let Some(sockets) = self.sockets {
            smp_string.push_str(&format!(",sockets={}", sockets));
        }

        if let Some(maxcpus) = self.maxcpus {
            smp_string.push_str(&format!(",maxcpus={}", maxcpus));
        }

        if !string.is_empty() {
            args.push("-cpu".to_string());
            args.push(string);
        }

        if !smp_string.is_empty() {
            args.push("-smp".to_string());
            args.push(smp_string);
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ManifestQemuMemory {
    size: Byte,
    slots: Option<usize>,
    maxmem: Option<Byte>,
}

impl ManifestQemuMemory {
    fn to_args(&self, args: &mut Vec<String>) {
        let mut string = String::new();

        string.push_str(&format!("size={}b", self.size));

        if let Some(slots) = self.slots {
            string.push_str(&format!(",slots={}", slots));
        }

        if let Some(maxmem) = self.maxmem {
            string.push_str(&format!(",maxmem={}", maxmem));
        }

        if !string.is_empty() {
            args.push("-m".to_string());
            args.push(string);
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ManifestQemuDrive {
    file: Url,
    interface: ManifestQemuDriveInterface,
    format: String,
    readonly: Option<bool>,
}

impl ManifestQemuDrive {
    async fn to_args(&self, args: &mut Vec<String>) -> Result<()> {
        let mut string = String::new();

        // TODO: resolve protocol
        string.push_str(&format!(
            "file={},",
            resolve(&self.file, None).await?.display()
        ));

        if self.interface != ManifestQemuDriveInterface::None {
            string.push_str(&format!("if={},", self.interface));
        }

        if let Some(readonly) = self.readonly {
            string.push_str(&format!("readonly={},", readonly));
        }

        string.push_str(&format!("format={}", self.format));

        if !string.is_empty() {
            args.push("-drive".to_string());
            args.push(string);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, serde::Deserialize)]
pub enum ManifestQemuDriveInterface {
    #[default]
    #[serde(rename = "none")]
    None,

    #[serde(rename = "ide")]
    IDE,

    #[serde(rename = "scsi")]
    SCSI,

    #[serde(rename = "sd")]
    SD,

    #[serde(rename = "mtd")]
    MTD,

    #[serde(rename = "floppy")]
    Floppy,

    #[serde(rename = "pflash")]
    Pflash,

    #[serde(rename = "virtio")]
    VirtIO,
}

impl Display for ManifestQemuDriveInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::None => write!(f, "none"),
            Self::IDE => write!(f, "ide"),
            Self::SCSI => write!(f, "scsi"),
            Self::SD => write!(f, "sd"),
            Self::MTD => write!(f, "mtd"),
            Self::Floppy => write!(f, "floppy"),
            Self::Pflash => write!(f, "pflash"),
            Self::VirtIO => write!(f, "virtio"),
        }
    }
}

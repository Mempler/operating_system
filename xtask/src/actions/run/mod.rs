use std::path::{Path, PathBuf};

use clap::Parser;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{manifest::Manifest, resolve::copy};

/// Boot
#[derive(Debug, Parser)]
pub struct RunOpt {
    #[clap(long, default_value = "config/manifest.yaml")]
    manifest: PathBuf,

    #[clap(short, long)]
    verbose: bool,
}

/// Error type for run actions.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[error(transparent)]
    Qemu(#[from] crate::manifest::ManifestQemuError),

    #[error(transparent)]
    Disk(#[from] crate::manifest::ManifestDiskError),

    #[error(transparent)]
    Resolve(#[from] crate::resolve::ResolveError),
}

/// Result type for build actions.
type Result<T, E = RunError> = std::result::Result<T, E>;

/// Perform the build action.
pub async fn perform(opt: RunOpt) -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(if opt.verbose {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        })
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let mut manifest_file = File::open(opt.manifest).await?;
    let mut manifest_yaml = String::new();

    manifest_file.read_to_string(&mut manifest_yaml).await?;

    let manifest: Manifest = serde_yaml::from_str(&manifest_yaml)?;

    let output = manifest.disk.build(Path::new("target/disk")).await?;
    let target = Path::new("target/disk.img");

    copy(&output, target, true).await?;

    manifest.qemu.exec().await?;

    Ok(())
}

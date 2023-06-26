use std::path::{Path, PathBuf};

use heck::ToPascalCase;
use tokio::fs::symlink;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Unimplemented Scheme: {0}")]
    UnimplementedScheme(String),

    #[error("The query parameter {0} is invalid, it requires {1}")]
    InvalidQueryParameter(String, String),

    #[error("Failed to build {0}, exit code {1}")]
    BuildFailed(String, i32),
}

type Result<T, E = ResolveError> = std::result::Result<T, E>;

/// Resolve a URL to a local path.
///
/// This will build each of the supported schemes and return a local path.
///
/// # Arguments
///
/// * `src` - The URL to resolve.
/// * `dst` - The destination path to copy the resolved file to.
///
/// # Returns
///
/// The local path to the resolved file.
///
/// # Errors
///
/// * `ResolveError::UnimplementedScheme` - The scheme is not supported.
/// * `ResolveError::InvalidScheme` - The scheme is invalid.
/// * `ResolveError::Io` - An IO error occured.
///
pub async fn resolve(src: &Url, dst: Option<&Path>) -> Result<PathBuf> {
    let result = match src.scheme() {
        "file" => process_scheme_file(&src, dst).await,
        "cargo" => process_scheme_cargo(&src, dst).await,
        "zig" => process_scheme_zig(&src, dst).await,

        _ => Err(ResolveError::UnimplementedScheme(src.scheme().to_string())),
    };

    trace!("Resolving {} -> {:?}", src, result);

    result
}

/// Process `file://` URLs.
///
async fn process_scheme_file(src: &Url, dst: Option<&Path>) -> Result<PathBuf> {
    let src = PathBuf::from(format!("{}{}", src.host().unwrap(), src.path()));

    if dst.is_none() {
        return Ok(src);
    }

    copy(&src, &dst.unwrap(), true).await?;

    Ok(dst.unwrap().to_path_buf())
}

/// Process `cargo://` URLs.
///
async fn process_scheme_cargo(src: &Url, dst: Option<&Path>) -> Result<PathBuf> {
    let crate_dir = PathBuf::from(format!("{}{}", src.host().unwrap(), src.path()));
    let crate_name = crate_dir.file_stem().unwrap();

    info!("Building {crate_name:?}");
    trace!("Dir: {crate_dir:#?}");

    let mut q = src.query_pairs();

    // TODO! check if target is valid

    let target = query(&mut q, "target", "x86_64-unknown-linux-gnu", &[])?;
    let profile = query(&mut q, "profile", "release", &["debug", "release"])?;
    let features = query(&mut q, "features", "", &[])?;

    let code = tokio::process::Command::new("cargo")
        .args(&[
            "build",
            "--target",
            &target,
            "--profile",
            &profile.replace("dev", "debug"),
            "--features",
            &features,
        ])
        .current_dir(&crate_dir)
        .status()
        .await?;

    if !code.success() {
        return Err(ResolveError::BuildFailed(
            crate_name.to_string_lossy().to_string(),
            code.code().unwrap(),
        ));
    }

    let src = crate_dir
        .join("target")
        .join(target)
        .join(profile)
        .join(crate_name);

    if dst.is_none() {
        return Ok(src);
    }

    copy(&src, &dst.unwrap(), true).await?;

    Ok(dst.unwrap().to_path_buf())
}

/// Process `zig://` URLs.
///
async fn process_scheme_zig(src: &Url, dst: Option<&Path>) -> Result<PathBuf> {
    let source_dir = PathBuf::from(format!("{}{}", src.host().unwrap(), src.path()));
    let source_name = source_dir.file_stem().unwrap();

    info!("Building {source_name:?} :: {source_dir:#?}");

    let optimize = query(
        &mut src.query_pairs(),
        "optimize",
        "release_fast",
        &["debug", "release_safe", "release_fast", "release_small"],
    )?;

    let code = tokio::process::Command::new("zig")
        .args(&[
            "build",
            &format!("-Doptimize={}", optimize.to_pascal_case()),
        ])
        .current_dir(&source_dir)
        .status()
        .await?;

    if !code.success() {
        return Err(ResolveError::BuildFailed(
            source_name.to_string_lossy().to_string(),
            code.code().unwrap(),
        ));
    }

    let src = source_dir.join("zig-out").join("bin").join(source_name);

    if dst.is_none() {
        return Ok(src);
    }

    copy(&src, &dst.unwrap(), true).await?;

    Ok(dst.unwrap().to_path_buf())
}

// Helpers to prevent code duplication
// -----------------------------------------------------------------------------

/// Copy a file from `src` to `dst`.
///
/// This will create the parent directory of `dst` if it does not exist.
///
/// # Arguments
///
/// * `src` - The source path to copy from.
/// * `dst` - The destination path to copy to.
///
/// # Errors
///
/// * `ResolveError::Io` - An IO error occured.
///
pub async fn copy(src: &Path, dst: &Path, do_symlink: bool) -> Result<()> {
    trace!(
        "Copying {src} to {dst}",
        src = src.display(),
        dst = dst.display()
    );

    tokio::fs::create_dir_all(dst.parent().unwrap()).await?;

    if do_symlink {
        let full_path = std::fs::canonicalize(src)?;

        if dst.is_symlink() || dst.exists() {
            std::fs::remove_file(&dst)?;
        }

        symlink(full_path, dst).await?;
    } else {
        tokio::fs::copy(src, dst).await?;
    }

    Ok(())
}

/// Query a URL for a value.
///
/// If the value is not present, the default value will be used.
///
/// If the value is present, it will be checked against the valid values.
///
/// # Arguments
///
/// * `query` - The query to search.
/// * `what` - The key to search for.
/// * `default` - The default value to use if the key is not present.
/// * `valid` - The valid values for the key.
///
/// # Returns
///
/// The value of the key.
///
/// # Errors
///
/// * `ResolveError::InvalidScheme` - The value is not valid.
///
fn query(
    query: &mut url::form_urlencoded::Parse<'_>,
    key: &str,
    default: &str,
    valid: &[&str],
) -> Result<String> {
    let result = query
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.to_string())
        .unwrap_or(String::from(default));

    trace!("Resolved {key} to {result}");

    if !valid.is_empty() && !valid.contains(&result.as_str()) {
        return Err(ResolveError::InvalidQueryParameter(
            key.to_string(),
            valid.join("|"),
        ));
    }

    Ok(result)
}

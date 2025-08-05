use std::fs::{self, File};
use std::path::Path;
use tar::{Builder, Header};
use anyhow::{Result, Context};
use std::io::{self, Read, Write};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use vantara::{safe_eprintln, safe_println, package_name};

/// Masukkan fail atau direktori secara auto. Verbose = true untuk log.
pub fn append_auto<B: io::Write>(
    builder: &mut Builder<B>,
    path: &Path,
    verbose: bool,
) -> Result<()> {
    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("{}: cannot get metadata from '{}'", package_name!(), path.display()))?;

    if meta.is_file() {
        append_file(builder, path, verbose)?;
    } else if meta.is_dir() {
        append_dir(builder, path, verbose)?;
    } else {
        if verbose {
            safe_eprintln(format_args!("{}: [SKIP] {:?} not file or directory", package_name!(), path));
        }
    }

    Ok(())
}

/// Extract archive automatik mengikut format
pub fn extract_auto(
    archive_path: &Path,
    target_dir: &Path,
    verbose: bool,
) -> Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("{}: failed to open file '{}'", package_name!(), archive_path.display()))?;

    let ext = archive_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let reader: Box<dyn Read> = match ext {
        "gz" => Box::new(GzDecoder::new(file)),
        "bz2" => Box::new(BzDecoder::new(file)),
        "xz" => Box::new(XzDecoder::new(file)),
        _     => Box::new(file),
    };

    let mut archive = tar::Archive::new(reader);
    archive.unpack(target_dir)
        .with_context(|| format!("{}: failed extract to '{}'", package_name!(), target_dir.display()))?;

    if verbose {
        safe_println(format_args!("{}: done extract: {} → {}", package_name!(), archive_path.display(), target_dir.display()));
    }

    Ok(())
}

/// Masukkan fail dengan header custom (tanpa owner/group/time)
fn append_file<B: io::Write>(
    builder: &mut Builder<B>,
    path: &Path,
    verbose: bool,
) -> Result<()> {
    let mut file = File::open(path)?;
    let meta = file.metadata()?;
    let mut header = Header::new_gnu();
    header.set_metadata(&meta);
    header.set_path(path)?;
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_username("root")?;
    header.set_groupname("root")?;
    header.set_cksum();

    if verbose {
        safe_println(format_args!("{}: [FILE] {}", package_name!(), path.display()));
    }

    builder.append(&header, &mut file)
        .with_context(|| format!("{}: failed to add file '{}'", package_name!(), path.display()))
}

/// Masukkan direktori secara rekursif
fn append_dir<B: io::Write>(
    builder: &mut Builder<B>,
    dir: &Path,
    verbose: bool,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        append_auto(builder, &path, verbose)?;
    }

    // Tambah direktori itu sendiri
    builder.append_dir_all(dir, dir)?;
    if verbose {
        safe_println(format_args!("{}: [DIR ] {}", package_name!(), dir.display()));
    }

    Ok(())
}

pub fn create_archive(
    out_path: &Path,
    inputs: &[&str],
    compression: Option<&str>,
    verbose: bool,
) -> Result<()> {
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(out_path)?;
    let buf = BufWriter::new(file);

    let boxed: Box<dyn Write> = match compression {
        Some("gz") => Box::new(GzEncoder::new(buf, flate2::Compression::default())),
        Some("bz2") => Box::new(BzEncoder::new(buf, bzip2::Compression::default())),
        Some("xz") => Box::new(XzEncoder::new(buf, 6)),
        _ => Box::new(buf),
    };

    let mut builder = Builder::new(boxed);

    for input in inputs {
        let path = Path::new(input);
        append_auto(&mut builder, path, verbose)?;
    }

    builder.finish()?;
    if verbose {
        safe_println(format_args!("{}: Successfully create archive '{}'", package_name!(), out_path.display()));
    }

    Ok(())
}


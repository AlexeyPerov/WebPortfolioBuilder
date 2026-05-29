use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

pub fn prepare_destination(target_dir: &Path) -> io::Result<()> {
    if target_dir.exists() {
        fs::remove_dir_all(target_dir)?;
    }
    Ok(())
}

pub fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    Ok(())
}

pub fn copy_template_static_assets(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry
            .path()
            .extension()
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("html"))
        {
            copy_file(entry.path(), &target)?;
        }
    }
    Ok(())
}

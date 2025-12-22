use crate::error::Result;
use std::{fs, io::ErrorKind, path::Path};

pub fn ensure_exists(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        r => r,
    }
    .expect(&format!(
        "Failed to ensure existence at at '{}'",
        path.display()
    ));
    Ok(())
}

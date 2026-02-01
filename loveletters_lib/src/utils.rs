use crate::error::{Error, Result};
use std::{fs, io::ErrorKind, path::Path};

pub fn ensure_exists(path: &Path) -> Result<()> {
    match fs::create_dir(path) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        r => r,
    }
    .map_err(|e| Error::FileIO {
        path: Some(path.to_path_buf()),
        raw: e,
    })?;
    Ok(())
}

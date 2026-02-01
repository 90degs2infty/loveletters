//! Error handling

use std::{fmt::Debug, io::Error as IoError, path::PathBuf};
use thiserror::Error;

/// Entities within a `loveletters` project
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum EntityKind {
    /// Toplevel input directory containing a self-contained `loveletters` project
    InputDirectory,
    /// Toplevel output directory for compiled content
    OutputDirectory,
    /// Toplevel project configuration file
    ProjectConfig,
    /// A page's root content file
    TypstRoot,
    /// Some unspecified entity
    Other,
}

impl EntityKind {
    fn describe(&self) -> &'static str {
        match self {
            EntityKind::InputDirectory => "input directory",
            EntityKind::OutputDirectory => "output directory",
            EntityKind::ProjectConfig => "project configuration file",
            EntityKind::TypstRoot => "typst root file",
            EntityKind::Other => "file or directory",
        }
    }
}

fn build_desc_fileio(path: &Option<PathBuf>) -> String {
    match path {
        None => "".to_string(),
        Some(path) => format!(" for path '{}'", path.display()),
    }
}

fn fmt_source_chain(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{:?}", cause)?;
        current = cause.source();
    }
    Ok(())
}

/// Failure conditions encountered during project compilation
#[derive(Error)]
#[non_exhaustive]
pub enum Error {
    /// File or directory not found
    #[error("failed to find {entity} at '{path}'", entity = missing.describe())]
    NotFound {
        /// The entity that is missing
        missing: EntityKind,
        /// The path that got searched for the missing entity
        path: PathBuf,
    },
    /// Invalid slug
    #[error("failed to derive slug for path '{path}'")]
    InvalidSlug {
        /// The erroneous filesystem path
        path: PathBuf,
    },
    /// Arbitrary file IO error
    // ISSUE(7): is there a way to avoid the allocation when building this error message?
    #[error("failed to perform file IO{desc}", desc = build_desc_fileio(&path))]
    FileIO {
        /// The path associated with the underlying error
        path: Option<PathBuf>,
        /// The underlying error
        #[source]
        raw: IoError,
    },
    /// Malformed toplevel project config
    #[error("failed to parse the project configuration from '{location}'")]
    MalformedProjectConfig {
        /// The erroneous project configuration's filesystem location
        location: PathBuf,
        #[source]
        /// The underlying error
        raw: anyhow::Error,
    },
    /// Malformed frontmatter
    #[error("failed to parse frontmatter from '{location}'")]
    MalformedFrontmatter {
        /// The erroneous frontmatter's filesystem location
        location: PathBuf,
        /// The underlying error
        #[source]
        raw: anyhow::Error,
    },
    /// Malformed project structure
    #[error("detected malformed project structure at '{path}'")]
    MalformedProjectStructure {
        /// The project structure violating path
        path: PathBuf,
    },
    /// Typst compilation failed
    #[error("failed to compile content of page at '{page}'")]
    Compilation {
        /// The erroneous content page
        page: PathBuf,
        /// The underlying error
        #[source]
        raw: anyhow::Error,
    },
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_source_chain(&self, f)
    }
}

/// Default return type for fallible operations
pub type Result<T> = std::result::Result<T, Error>;

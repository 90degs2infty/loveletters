//! Error handling

use std::{
    error,
    fmt::{self, Debug, Display},
    io::{Error as IoError, ErrorKind as IoKind},
    path::{Path, PathBuf},
    result,
};
use thiserror::Error;

/// Entities within a `loveletters` project
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum EntityKind {
    /// Toplevel input directory containing a self-contained `loveletters` project
    InputDirectory,
    /// Project subdirectory containing the content-tree
    ContentDirectory,
    /// Toplevel output directory for compiled content
    OutputDirectory,
    /// Toplevel project configuration file
    ProjectConfig,
    /// A page's root content file
    TypstRoot,
    /// Toplevel section index frontmatter file
    ToplevelSectionIndex,
    /// Some unspecified entity
    Other,
}

impl Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityKind::InputDirectory => write!(f, "input directory"),
            EntityKind::ContentDirectory => write!(f, "content directory"),
            EntityKind::OutputDirectory => write!(f, "output directory"),
            EntityKind::ProjectConfig => write!(f, "project configuration file"),
            EntityKind::TypstRoot => write!(f, "typst root file"),
            EntityKind::ToplevelSectionIndex => {
                write!(f, "toplevel section index page frontmatter")
            }
            EntityKind::Other => write!(f, "file or directory"),
        }
    }
}

fn build_desc_fileio(path: Option<&Path>) -> String {
    match path {
        None => String::new(),
        Some(path) => format!(" at path '{}'", path.display()),
    }
}

fn fmt_source_chain(e: &impl error::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{e}")?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "\nCaused by:\n\t{cause:?}")?;
        current = cause.source();
    }
    Ok(())
}

/// Failure conditions encountered during project compilation
#[derive(Error)]
#[non_exhaustive]
#[must_use]
pub enum Error {
    /// File or directory not found
    // ISSUE(7): is there a way to avoid the allocation when building this error message?
    #[error("failed to find {missing}{desc}", desc = build_desc_fileio(path.as_deref()))]
    NotFound {
        /// The entity that is missing
        missing: EntityKind,
        /// The path that got searched for the missing entity
        path: Option<PathBuf>,
    },
    /// Invalid slug
    #[error("failed to derive slug for path '{path}'")]
    InvalidSlug {
        /// The erroneous filesystem path
        path: PathBuf,
    },
    /// Arbitrary file IO error
    // ISSUE(7): is there a way to avoid the allocation when building this error message?
    #[error("failed to perform file IO{desc}", desc = build_desc_fileio(path.as_deref()))]
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
    #[error("failed to parse frontmatter")]
    MalformedFrontmatter {
        /// The underlying error
        #[source]
        raw: Box<dyn error::Error>,
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
    /// Decorator indicating the erroneous path
    #[error("while working on {path}")]
    AtPath {
        /// The file who's processing eventually lead to the wrapped error.
        path: PathBuf,
        /// The source error.
        #[source]
        raw: Box<Self>,
    },
}

// TODO: switch to an error layout more similar to what std::io::Error does.

impl Error {
    /// Turn the specified [`IoError`] into an [`enum@Error`].
    // TODO: Seal calls to this function?
    pub fn from_io_error(e: IoError, path: Option<PathBuf>, entity: EntityKind) -> Self {
        match e.kind() {
            IoKind::NotFound => Error::NotFound {
                missing: entity,
                path,
            },
            _ => Error::FileIO { path, raw: e },
        }
    }

    /// Attach additional information on where the error occurred in terms of file system location.
    ///
    /// Calling [`Error::at`] with `path` carries the semantics of `self` having resulted from the processing of `path`.
    pub fn at(self, path: PathBuf) -> Self {
        Self::AtPath {
            path,
            raw: Box::new(self),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_source_chain(&self, f)
    }
}

/// Default return type for fallible operations
pub type Result<T> = result::Result<T, Error>;

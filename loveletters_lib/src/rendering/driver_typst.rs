// This module has been adapted from https://github.com/tfachmann/typst-as-library/blob/7710cc196465d99e8bbeff54f8dee229ff612cb0/src/lib.rs,
// released under Apache License, Version 2.0, (license available at https://github.com/tfachmann/typst-as-library/blob/7710cc196465d99e8bbeff54f8dee229ff612cb0/LICENSE).
//
// Changes include:
// - annotations of future todos
// - adaptions in the way `TypstEngine` (formerly known as `TypstWrapperWorld`) is instantiated
// - lookup procedure for packages from the `loveletters` namespace

use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use typst::diag::{FileError, FileResult, PackageError, PackageResult, eco_format};
use typst::foundations::{Bytes, Datetime, Dict, IntoValue};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Feature, Library, LibraryExt};
use typst_kit::fonts::{FontSearcher, FontSlot};
use ureq::agent;

use crate::{
    error::{EntityKind, Error, Result as CrateResult},
    rendering::context::{PageContext, ProjectContext},
};

// TODO: check that pathbuf actually is relative (maybe use VirtualPath instead?)!
type RelativePath = PathBuf;

// TODO there is probably quite a lot you can reuse across invocations of typst for different
// posts.
/// Main interface that determines the environment for Typst.
pub struct TypstEngine {
    /// Root path to which files will be resolved.
    root: PathBuf,

    /// The content of a source.
    source: Source,

    /// The standard library.
    library: LazyHash<Library>,

    /// Metadata about all known fonts.
    book: LazyHash<FontBook>,

    /// Metadata about all known fonts.
    fonts: Vec<FontSlot>,

    /// Map of all known files.
    files: Arc<Mutex<HashMap<FileId, FileEntry>>>,

    /// Cache directory (e.g. where packages are downloaded to).
    cache_directory: PathBuf,

    /// Project package directory.
    ///
    /// Packages from the `loveletters` namespace are loaded from this directory.
    project_packages_directory: PathBuf,

    // TODO maybe use `download` from `typst_kit` or `reqwest` instead?
    /// http agent to download packages.
    http: ureq::Agent,

    // TODO maybe use a different time library for this?
    /// Datetime.
    time: time::OffsetDateTime,
}

impl TypstEngine {
    pub fn new(
        root_dir: PathBuf,
        root_file: RelativePath,
        project_packages_directory: PathBuf,
        gctx: ProjectContext,
        pctx: PageContext,
    ) -> CrateResult<Self> {
        let root_file = root_dir.join(root_file);
        // Top-level content and directory
        println!("Working on {}", root_file.display());

        let root_src = std::fs::read_to_string(&root_file).map_err(|e| match e.kind() {
            ErrorKind::NotFound => Error::NotFound {
                missing: EntityKind::TypstRoot,
                path: root_file,
            },
            _ => Error::FileIO {
                path: Some(root_file),
                raw: e,
            },
        })?;

        // Library
        let mut lib = Library::builder()
            .with_features([Feature::Html].into_iter().collect())
            .build();

        let fonts = FontSearcher::new().include_system_fonts(true).search();

        // Inject loveletters' default top-level bindings
        let mut ctx = Dict::new();
        ctx.insert("project".into(), gctx.into_value());
        ctx.insert("page".into(), pctx.into_value());

        lib.global
            .scope_mut()
            .define("loveletters", ctx.into_value());

        Ok(Self {
            library: LazyHash::new(lib),
            book: LazyHash::new(fonts.book),
            root: root_dir,
            fonts: fonts.fonts,
            source: Source::detached(root_src),
            time: time::OffsetDateTime::now_utc(),
            // TODO set env-dir using proper config handling (e.g. `config` crate)
            // TODO reuse across instantiations of `TypstEngine` to reduce the number of package downloads
            cache_directory: std::env::var_os("CACHE_DIRECTORY")
                .map(|os_path| os_path.into())
                .unwrap_or(std::env::temp_dir()),
            project_packages_directory,
            http: agent(),
            files: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
        Self {
            bytes: Bytes::new(bytes),
            source,
        }
    }

    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

impl TypstEngine {
    /// Helper to handle file requests.
    ///
    /// Requests will be either in packages or a local file.
    fn file(&self, id: FileId) -> FileResult<FileEntry> {
        let mut files = self.files.lock().map_err(|_| FileError::AccessDenied)?;
        if let Some(entry) = files.get(&id) {
            return Ok(entry.clone());
        }
        let path = if let Some(package) = id.package() {
            // Fetching file from package
            let package_dir = self.lookup_or_download_package(package)?;
            id.vpath().resolve(&package_dir)
        } else {
            // Fetching file from disk
            id.vpath().resolve(&self.root)
        }
        .ok_or(FileError::AccessDenied)?;

        let content = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(files
            .entry(id)
            .or_insert(FileEntry::new(content, None))
            .clone())
    }

    /// Lookup packages from the `loveletters` namespace
    fn lookup_project_package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        if package.name == "loveletters" {
            // TODO
            // Currently, we expect the user to copy/link the loveletters package. At a later
            // point, we will want to provide it ourselves.
        }

        let package_dir = self.project_packages_directory.join(package.name.as_str());
        if !package_dir.exists() {
            return Err(PackageError::NotFound(package.clone()));
        }

        let version_dir = package_dir.join(format!("{}", package.version));
        if !version_dir.exists() {
            return Err(PackageError::VersionNotFound(
                package.clone(),
                package.version,
            ));
        }

        Ok(version_dir)
    }

    /// Downloads the package and returns the system path of the unpacked package.
    fn lookup_or_download_package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        if package.namespace == "loveletters" {
            return self.lookup_project_package(package);
        }

        let package_subdir = format!("{}/{}/{}", package.namespace, package.name, package.version);

        let path = self.cache_directory.join(package_subdir);

        if path.exists() {
            return Ok(path);
        }

        eprintln!("downloading {package}");
        let url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            package.namespace, package.name, package.version,
        );

        let mut response = retry(|| {
            let response = self
                .http
                .get(&url)
                .call()
                .map_err(|error| eco_format!("{error}"))?;

            let status = response.status();
            if !http_successful(status.into()) {
                return Err(eco_format!(
                    "response returned unsuccessful status code {status}",
                ));
            }

            Ok(response)
        })
        .map_err(|error| PackageError::NetworkFailed(Some(error)))?;

        let mut compressed_archive = Vec::new();
        response
            .body_mut()
            .as_reader()
            .read_to_end(&mut compressed_archive)
            .map_err(|error| PackageError::NetworkFailed(Some(eco_format!("{error}"))))?;
        // TODO maybe look into alternatives to zune_inflate?
        let raw_archive = zune_inflate::DeflateDecoder::new(&compressed_archive)
            .decode_gzip()
            .map_err(|error| PackageError::MalformedArchive(Some(eco_format!("{error}"))))?;
        let mut archive = tar::Archive::new(raw_archive.as_slice());
        archive.unpack(&path).map_err(|error| {
            _ = std::fs::remove_dir_all(&path);
            PackageError::MalformedArchive(Some(eco_format!("{error}")))
        })?;

        Ok(path)
    }
}

/// This is the interface we have to implement such that `typst` can compile it.
///
/// I have tried to keep it as minimal as possible
impl typst::World for TypstEngine {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.file(id)?.source(id)
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.file(id).map(|file| file.bytes.clone())
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

fn retry<T, E>(mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    if let Ok(ok) = f() { Ok(ok) } else { f() }
}

fn http_successful(status: u16) -> bool {
    // 2XX
    status / 100 == 2
}

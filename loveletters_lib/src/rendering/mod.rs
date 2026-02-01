pub mod context;
mod driver_typst;

use std::{marker::PhantomData, path::PathBuf};
use typst::{
    diag::{Severity, SourceDiagnostic},
    ecow::EcoVec,
};

use crate::{
    bundleing::{InMemFile, PageBundle},
    content::{IndexFrontmatter, LeafFrontmatter},
    error::{Error, Result},
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf, Mode},
    rendering::context::{PageContext, ProjectContext},
    section::Section,
};
use driver_typst::TypstEngine;
use typst_html::HtmlDocument;

struct TypstError {
    diagnostics: EcoVec<SourceDiagnostic>,
}

impl TypstError {
    fn from_diagnostics(diagnostics: EcoVec<SourceDiagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl From<EcoVec<SourceDiagnostic>> for TypstError {
    fn from(value: EcoVec<SourceDiagnostic>) -> Self {
        Self::from_diagnostics(value)
    }
}

// Here is how typst prints out messages internally (for a HintedStrResult -> HintedString):
// https://github.com/typst/typst/blob/bf946178ec3cb34c06b4666ea237a025b4ca4aa0/crates/typst-cli/src/main.rs#L59-L65
// https://docs.rs/typst/latest/typst/diag/type.HintedStrResult.html

impl std::fmt::Display for TypstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to compile document")
    }
}

fn describe_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

impl std::fmt::Debug for TypstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.diagnostics.len() > 0 {
            for d in &self.diagnostics {
                write!(f, "\n    {}: {}", describe_severity(d.severity), d.message)?;
                for h in &d.hints {
                    write!(f, "\n        > {}", h)?;
                }
            }
        } else {
            writeln!(
                f,
                "failed to compile document without further diagnostics to show"
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for TypstError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub struct RenderedPage<M> {
    content_dir: PathBuf,
    rendering: HtmlDocument,
    m: PhantomData<M>,
}

impl<M> RenderedPage<M> {
    pub fn new(content_dir: PathBuf, rendering: HtmlDocument) -> Self {
        Self {
            content_dir,
            rendering,
            m: PhantomData,
        }
    }

    pub fn try_bundle(self, output_dir: PathBuf) -> Result<PageBundle> {
        let html = typst_html::html(&self.rendering).map_err(|e| {
            let err: TypstError = e.into();
            Error::Compilation {
                page: self.content_dir,
                raw: err.into(),
            }
        })?;
        Ok(PageBundle::new(output_dir, InMemFile::new(html.into())))
    }
}

pub struct Renderer {
    ctx: ProjectContext,
    project_packages: PathBuf,
}

impl Renderer {
    pub fn new(ctx: ProjectContext, project_packages_dir: PathBuf) -> Self {
        Self {
            ctx,
            project_packages: project_packages_dir,
        }
    }

    pub fn try_render(
        &self,
        content: Section<
            PageWithFrontmatter<Index, IndexFrontmatter>,
            PageWithFrontmatter<Leaf, LeafFrontmatter>,
        >,
    ) -> Result<Section<RenderedPage<Index>, RenderedPage<Leaf>>> {
        content.try_walk(
            |path, page| {
                let ctx = PageContext::new(path, None);
                page.try_render(&self, ctx)
            },
            |path, slug, page| {
                let ctx = PageContext::new(path, Some(slug));
                page.try_render(&self, ctx)
            },
        )
    }

    pub fn try_render_dir<M>(
        &self,
        content_dir: PathBuf,
        page_ctx: PageContext,
    ) -> Result<RenderedPage<M>>
    where
        M: Mode,
    {
        // TODO: should probably be something like
        // let engine = TypstEngine::new();
        // let entrypoint = engine.wrap(&self);
        // or similar...
        let root_file = M::typst_filename().into();
        let entrypoint = TypstEngine::new(
            content_dir.clone(),
            root_file,
            self.project_packages.clone(),
            self.ctx.clone(),
            page_ctx,
        );
        let typst_document = typst::compile(&entrypoint?).output.map_err(|e| {
            let err: TypstError = e.into();
            Error::Compilation {
                page: content_dir.clone(),
                raw: err.into(),
            }
        })?;
        Ok(RenderedPage::new(content_dir, typst_document))
    }
}

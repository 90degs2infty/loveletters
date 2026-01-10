pub mod context;
mod driver_typst;

use std::{marker::PhantomData, path::PathBuf};

use crate::{
    bundleing::{InMemFile, PageBundle},
    content::{IndexFrontmatter, LeafFrontmatter},
    error::Result,
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf, Mode},
    rendering::context::{ProjectContext, PageContext},
    section::Section,
};
use driver_typst::TypstEngine;
use typst_html::HtmlDocument;

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
        let html = typst_html::html(&self.rendering).expect("Failed to export document to HTML.");
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
        let typst_document = typst::compile(&entrypoint)
            .output
            .expect("Failed to compile post using typst");
        Ok(RenderedPage::new(content_dir, typst_document))
    }
}

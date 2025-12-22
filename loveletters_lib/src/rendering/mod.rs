pub mod context;

use std::{marker::PhantomData, path::PathBuf};

use crate::{
    bundleing::{InMemFile, PageBundle},
    content::{IndexFrontmatter, LeafFrontmatter},
    driver_typst::TypstEngine,
    error::Result,
    frontmatter_parsing::PageWithFrontmatter,
    page::{Index, Leaf},
    rendering::context::Context,
    section::Section,
    slug::Slug,
    utils::ensure_exists,
};

pub struct RenderedPage<M> {
    // TODO drop
    slug: Slug,
    content_dir: PathBuf,
    rendering: String,
    m: PhantomData<M>,
}

impl<M> RenderedPage<M> {
    pub fn try_render(
        slug: Slug,
        content_dir: PathBuf,
        root_file: PathBuf,
        ctx: Context,
    ) -> Result<Self> {
        // TODO: should probably be something like
        // let engine = TypstEngine::new();
        // let entrypoint = engine.wrap(&self);
        // or similar...
        let entrypoint = TypstEngine::new(content_dir.clone(), root_file, ctx);
        let typst_document = typst::compile(&entrypoint)
            .output
            .expect("Failed to compile post using typst");
        let html = typst_html::html(&typst_document).expect("Failed to export document to HTML.");
        Ok(Self {
            slug,
            content_dir,
            rendering: html,
            m: PhantomData,
        })
    }

    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    pub fn try_bundle(self, output_dir: PathBuf) -> Result<PageBundle> {
        ensure_exists(&output_dir)?;
        Ok(PageBundle::new(
            output_dir,
            InMemFile::new(self.rendering.into()),
        ))
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn try_render(
        &self,
        content: Section<
            PageWithFrontmatter<Index, IndexFrontmatter>,
            PageWithFrontmatter<Leaf, LeafFrontmatter>,
        >,
    ) -> Result<Section<RenderedPage<Index>, RenderedPage<Leaf>>> {
        let tree = content.to_typst();

        content.try_walk(
            |path, page| {
                let ctx = Context::new(tree.clone(), path, None);
                page.try_render(ctx)
            },
            |path, slug, page| {
                let ctx = Context::new(tree.clone(), path, Some(slug));
                page.try_render(ctx)
            },
        )
    }
}

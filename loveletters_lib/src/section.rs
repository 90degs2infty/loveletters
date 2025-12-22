use std::collections::HashMap;

use typst::foundations::{Dict, IntoValue, Value};

use crate::{error::Result, slug::Slug};

pub struct Section<I, L> {
    // TODO drop
    slug: Slug,
    index: I,
    pages: HashMap<Slug, L>,
    sub_sections: HashMap<Slug, Section<I, L>>,
}

impl<I, L> Section<I, L> {
    pub fn new(
        slug: Slug,
        index: I,
        pages: HashMap<Slug, L>,
        sub_sections: HashMap<Slug, Section<I, L>>,
    ) -> Self {
        Self {
            slug,
            index,
            pages,
            sub_sections,
        }
    }

    /// Map this section.
    ///
    /// If you need access to a [`Section`]'s fully qualified path, see [`Section::try_walk`] instead.
    pub fn try_map<J, M, FIndex, FLeaf>(
        mut self,
        f_index: FIndex,
        f_leaf: FLeaf,
    ) -> Result<Section<J, M>>
    where
        FIndex: Fn(I) -> Result<J> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(L) -> Result<M> + Clone,
    {
        let new_index = f_index(self.index)?;
        let new_leafs = self
            .pages
            .drain()
            .map(|(slug, page)| (slug, f_leaf(page)))
            .map(|(slug, res)| Ok((slug, res?)))
            .collect::<Result<HashMap<_, _>>>()?;

        let new_subsecs = if !self.sub_sections.is_empty() {
            self.sub_sections
                .drain()
                .map(|(slug, sec)| (slug, sec.try_map(f_index.clone(), f_leaf.clone())))
                .map(|(slug, res)| Ok((slug, res?)))
                .collect::<Result<HashMap<_, _>>>()?
        } else {
            HashMap::new()
        };
        Ok(Section {
            slug: self.slug,
            index: new_index,
            pages: new_leafs,
            sub_sections: new_subsecs,
        })
    }

    fn try_walk_helper<J, M, FIndex, FLeaf>(
        mut self,
        path: Vec<Slug>,
        f_index: FIndex,
        f_leaf: FLeaf,
    ) -> Result<Section<J, M>>
    where
        FIndex: Fn(&[Slug], I) -> Result<J> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(&[Slug], &Slug, L) -> Result<M> + Clone,
    {
        let new_index = f_index(path.as_slice(), self.index)?;

        let new_leafs = self
            .pages
            .drain()
            .map(|(slug, page)| {
                let new_leaf = f_leaf(path.as_slice(), &slug, page);
                Ok((slug, new_leaf?))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let new_subsecs = if !self.sub_sections.is_empty() {
            self.sub_sections
                .drain()
                .map(|(slug, sec)| {
                    // TODO how to get rid of the excessive cloneing?
                    let mut path = path.clone();
                    path.push(slug.clone());
                    Ok((
                        slug,
                        sec.try_walk_helper(path, f_index.clone(), f_leaf.clone())?,
                    ))
                })
                .collect::<Result<HashMap<_, _>>>()?
        } else {
            HashMap::new()
        };
        Ok(Section {
            slug: self.slug,
            index: new_index,
            pages: new_leafs,
            sub_sections: new_subsecs,
        })
    }

    /// Walk this [`Section`].
    ///
    /// Walking a [`Section`] is similar to mapping it, except that the functions transforming the contained pages get access to additional context.
    /// Here the context is given by
    /// - the fully qualified section path (i.e. the sequence of [`Slug`]s) of this [`Section`] for this [`Section`]'s index page, as well as
    /// - the fully qualified section path alongside the respective page [`Slug`] for all leaf pages contained in this [`Section`].
    pub fn try_walk<J, M, FIndex, FLeaf>(
        self,
        f_index: FIndex,
        f_leaf: FLeaf,
    ) -> Result<Section<J, M>>
    where
        FIndex: Fn(&[Slug], I) -> Result<J> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(&[Slug], &Slug, L) -> Result<M> + Clone,
    {
        self.try_walk_helper(Vec::new(), f_index, f_leaf)
    }

    pub fn slug(&self) -> &Slug {
        &self.slug
    }
}

impl<I, L> Section<I, L>
where
    for<'a> &'a I: IntoValue,
    for<'b> &'b L: IntoValue,
{
    pub fn to_typst(&self) -> Dict {
        let Self {
            slug: _,
            index,
            pages,
            sub_sections,
        } = self;

        let mut pages_typst = Dict::new();
        pages.iter().for_each(|(slug, page)| {
            pages_typst.insert(slug.as_str().into(), page.into_value());
        });

        let mut sub_sections_typst = Dict::new();
        sub_sections.iter().for_each(|(slug, sec)| {
            sub_sections_typst.insert(slug.as_str().into(), Value::Dict(sec.to_typst()));
        });

        let mut d = Dict::new();
        d.insert("index".into(), index.into_value());
        d.insert("pages".into(), Value::Dict(pages_typst));
        d.insert("subsections".into(), Value::Dict(sub_sections_typst));
        d
    }
}

impl<'s, I, L> IntoValue for &'s Section<I, L>
where
    for<'a> &'a I: IntoValue,
    for<'b> &'b L: IntoValue,
{
    fn into_value(self) -> typst::foundations::Value {
        typst::foundations::Value::Dict(self.to_typst())
    }
}

use std::collections::HashMap;

use typst::foundations::{Dict, IntoValue, Value};

use crate::{error::Result, slug::Slug};

pub struct Section<P> {
    index: P,
    pages: HashMap<Slug, P>,
    sub_sections: HashMap<Slug, Section<P>>,
}

impl<P> Section<P> {
    pub fn new(index: P, pages: HashMap<Slug, P>, sub_sections: HashMap<Slug, Section<P>>) -> Self {
        Self {
            index,
            pages,
            sub_sections,
        }
    }

    /// Map this section.
    ///
    /// If you need access to a [`Section`]'s fully qualified path, see [`Section::try_walk`] instead.
    pub fn try_map<Q, FIndex, FLeaf>(mut self, f_index: FIndex, f_leaf: FLeaf) -> Result<Section<Q>>
    where
        FIndex: Fn(P) -> Result<Q> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(P) -> Result<Q> + Clone,
    {
        let new_index = f_index(self.index)?;
        let new_leafs = self
            .pages
            .drain()
            .map(|(slug, page)| (slug, f_leaf(page)))
            .map(|(slug, res)| Ok((slug, res?)))
            .collect::<Result<HashMap<_, _>>>()?;

        let new_subsecs = if self.sub_sections.is_empty() {
            HashMap::new()
        } else {
            self.sub_sections
                .drain()
                .map(|(slug, sec)| (slug, sec.try_map(f_index.clone(), f_leaf.clone())))
                .map(|(slug, res)| Ok((slug, res?)))
                .collect::<Result<HashMap<_, _>>>()?
        };
        Ok(Section {
            index: new_index,
            pages: new_leafs,
            sub_sections: new_subsecs,
        })
    }

    #[allow(
        clippy::needless_pass_by_value,
        reason = "path is cloned multiple times inside this function so do not pretend we do not need ownership"
    )]
    fn try_walk_helper<Q, FIndex, FLeaf>(
        mut self,
        path: Vec<Slug>,
        f_index: FIndex,
        f_leaf: FLeaf,
    ) -> Result<Section<Q>>
    where
        FIndex: Fn(&[Slug], P) -> Result<Q> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(&[Slug], &Slug, P) -> Result<Q> + Clone,
    {
        let new_index = f_index(&path, self.index)?;

        let new_leafs = self
            .pages
            .drain()
            .map(|(slug, page)| {
                let new_leaf = f_leaf(&path, &slug, page);
                Ok((slug, new_leaf?))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let new_subsecs = if self.sub_sections.is_empty() {
            HashMap::new()
        } else {
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
        };
        Ok(Section {
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
    pub fn try_walk<Q, FIndex, FLeaf>(self, f_index: FIndex, f_leaf: FLeaf) -> Result<Section<Q>>
    where
        FIndex: Fn(&[Slug], P) -> Result<Q> + Clone, // Clone to prevent recursive type
        FLeaf: Fn(&[Slug], &Slug, P) -> Result<Q> + Clone,
    {
        self.try_walk_helper(Vec::new(), f_index, f_leaf)
    }
}

impl<P> Section<P>
where
    for<'a> &'a P: IntoValue,
{
    pub fn to_typst(&self) -> Dict {
        let Self {
            index,
            pages,
            sub_sections,
        } = self;

        let mut pages_typst = Dict::new();
        for (slug, page) in pages {
            pages_typst.insert(slug.as_str().into(), page.into_value());
        }

        let mut sub_sections_typst = Dict::new();
        for (slug, sec) in sub_sections {
            sub_sections_typst.insert(slug.as_str().into(), Value::Dict(sec.to_typst()));
        }

        let mut d = Dict::new();
        d.insert("index".into(), index.into_value());
        d.insert("pages".into(), Value::Dict(pages_typst));
        d.insert("subsections".into(), Value::Dict(sub_sections_typst));
        d
    }
}

impl<P> IntoValue for &'_ Section<P>
where
    for<'a> &'a P: IntoValue,
{
    fn into_value(self) -> Value {
        Value::Dict(self.to_typst())
    }
}

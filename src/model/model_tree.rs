use indexmap::IndexMap;
use sal_3dlib::topology::Shape;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
///
/// Internal structure of [Ship] represented as tree.
///
/// [Ship]: crate::model::Ship
pub(super) struct ModelTree<A> {
    dbgid: DbgId,
    path: PathBuf,
    inner: IndexMap<String, Shape<Option<A>>>,
}
//
//
impl<A> ModelTree<A> {
    ///
    /// Creates a new instance of [ModelTree].
    pub(super) fn new(parent: &DbgId, path: impl AsRef<Path>) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "ModelTree.try_new"),
            path: path.as_ref().to_path_buf(),
            inner: IndexMap::new(),
        }
    }
    ///
    /// Builds the new instance of [ModelTree].
    ///
    /// Internally it reads `self.path` and converts the result to the target representation.
    pub(super) fn build(self) -> Result<Self, StrErr> {
        sal_3dlib::fs::Reader::read_step(&self.path)
            .map_err(|why| {
                StrErr(format!(
                    "{} | Failed reading model_path='{}': {}",
                    self.dbgid,
                    self.path.display(),
                    why
                ))
            })
            .and_then(|reader| {
                reader.into_vec::<A>().map_err(|why| {
                    StrErr(format!(
                        "{} | Failed reading model tree: {:?}",
                        self.dbgid, why
                    ))
                })
            })
            .map(|models| Self {
                inner: models.into_iter().collect(),
                ..self
            })
    }
}
//
//
impl<A> Deref for ModelTree<A> {
    type Target = IndexMap<String, Shape<Option<A>>>;
    //
    //
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
//
//
impl<A> DerefMut for ModelTree<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

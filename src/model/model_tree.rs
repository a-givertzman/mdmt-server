use indexmap::IndexMap;
use sal_3dlib::topology::Shape;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::path::{Path, PathBuf};
///
/// Internal structure of [super::ShipModel] represented as tree.
#[derive(Clone)]
pub(super) struct ModelTree<A> {
    //
    //
    dbgid: DbgId,
    ///
    /// Source file.
    path: PathBuf,
    ///
    /// Actual model representation.
    ///
    /// In the current version the model is considered to have one or more parts.
    /// If a part has a name, this name concatinated with the full path from the root is used as the key.
    /// The model part itself becomes the value of the key.
    models: IndexMap<String, Shape<Option<A>>>,
}
//
//
impl<A> ModelTree<A> {
    ///
    /// Creates a new instance.
    pub(super) fn new(parent: &DbgId, path: impl AsRef<Path>) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "ModelTree.new"),
            path: path.as_ref().to_path_buf(),
            models: IndexMap::new(),
        }
    }
    ///
    /// Builds the new instance.
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
                models: models.into_iter().collect(),
                ..self
            })
    }
    ///
    /// Return an iterator over the key-value pairs of the map, in their order.
    pub(super) fn iter(&self) -> indexmap::map::Iter<'_, String, Shape<Option<A>>> {
        self.models.iter()
    }
    ///
    /// Return a reference to the value stored for `key`, if it is present, else `None`.
    pub(super) fn get(&self, key: impl AsRef<str>) -> Option<&Shape<Option<A>>> {
        self.models.get(key.as_ref())
    }
    ///
    /// Return `true` if an equivalent to `key` exists in the map.
    pub(super) fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.models.contains_key(key.as_ref())
    }
}

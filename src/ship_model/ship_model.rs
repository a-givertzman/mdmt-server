//
use cache::{
    floating_position_cache::{FloatingPositionCache, FloatingPositionCacheConf},
    LocalCache,
};
use indexmap::{IndexMap, IndexSet};
use model_tree::ModelTree;
use sal_3dlib::{
    ops::boolean::volume::AlgoMakerVolume,
    props::Center,
    topology::{
        shape::{Compound, Face},
        Shape,
    },
};
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::{path::PathBuf, sync::Arc};
///
/// Defines relative position against an object.
///
/// Considered to be used to filter out [ShipModel] parts.
/// See [ShipModel::subvolume] to find an example of use.
pub enum RelativePostion {
    Above,
    Under,
}
///
/// Ship object being model with the attribute of type `A`.
///
/// See [sal_3dlib::props::Attributes] to get more details about what the attribute type is.
pub struct ShipModel<A> {
    //
    //
    dbgid: DbgId,
    ///
    /// Inner model structure.
    model_tree: ModelTree<A>,
    ///
    /// Collection of all caches used by the model.
    caches: IndexMap<CacheKey, Box<dyn LocalCache>>,
}
//
//
impl<A: Clone + Send + 'static> ShipModel<A> {
    ///
    /// Creates a new instance.
    pub fn new(parent: &DbgId, conf: ShipModelConf) -> Self {
        let dbgid = DbgId::with_parent(parent, "ShipModel");
        let model_tree = ModelTree::new(&dbgid, conf.model_path);
        let mut ship_model = Self {
            caches: IndexMap::new(),
            model_tree: model_tree.clone(),
            dbgid: dbgid.clone(),
        };
        ship_model.caches.insert(
            CacheKey::FloatingPostion,
            Box::new(FloatingPositionCache::new(
                &dbgid,
                model_tree,
                conf.floating_position_cache_conf,
            )),
        );
        ship_model
    }
    ///
    /// Returns model parts touched by `waterline` and filtered against [RelativePostion].
    ///
    /// The algorithm uses those parts of the `self.model_tree`, which are specified in `keys`.
    /// If `keys` is empty it's considered to use all model parts.
    /// _Note_ that in the both cases only those parts are used, which types can make volume.
    /// In particular, these types are [Face]s, [Shell]s, and [Solid]s.
    ///
    /// [Shell]: sal_3dlib::topology::shape::Shell
    /// [Solid]: sal_3dlib::topology::shape::Solid
    pub fn subvolume(
        &self,
        keys: &[&str],
        waterline: &Face<Option<A>>,
        relative_position: RelativePostion,
    ) -> Result<Vec<Shape<Option<A>>>, StrErr> {
        let dbgid = DbgId(format!("{}.subvolume", self.dbgid));
        // pop up warning if a key is not present in `self.model_key`
        for key in keys {
            if !self.model_tree.contains_key(key) {
                log::warn!("{} | No model found for key='{}'", dbgid, key);
            }
        }
        // defines whether the key should be taken
        let should_volume = |key| keys.is_empty() || self.model_tree.contains_key(key);
        let [.., waterline_z] = waterline.center().point();
        self.model_tree
            .iter()
            .filter_map(|(key, model)| {
                Some(match model {
                    Shape::Face(model) if should_volume(key) => {
                        Compound::build([waterline, model], [], [])
                    }
                    Shape::Shell(model) if should_volume(key) => {
                        Compound::build([waterline], [model], [])
                    }
                    Shape::Solid(model) if should_volume(key) => {
                        Compound::build([waterline], [], [model])
                    }
                    _ => return None,
                })
            })
            .try_fold(vec![], |mut shapes, build| {
                let model_part = build?;
                let [.., model_part_z] = model_part.center().point();
                if match relative_position {
                    RelativePostion::Above => model_part_z > waterline_z,
                    RelativePostion::Under => model_part_z < waterline_z,
                } {
                    shapes.push(Shape::Compound(model_part));
                }
                Ok(shapes)
            })
    }
    ///
    /// Generates and reload the internal caches.
    ///
    /// The field `caches` contains cache keys to update.
    /// Remaining it empty builds and reloads all the caches.
    ///
    /// # Errors
    /// Internally it creates worker threads while building.
    /// The result error is a collection of all failed worker errors joined by '\n'.
    pub fn update_caches(&mut self, caches: &[&CacheKey]) -> Result<(), StrErr> {
        // start wokers to calculate required caches
        let handlers = {
            let mut handlers = vec![];
            let calculate_all = caches.is_empty();
            for (cache_key, cache) in &self.caches {
                if calculate_all || caches.contains(&cache_key) {
                    cache
                        .calculate(Arc::default())
                        .map(|workers| handlers.push((*cache_key, workers)))?;
                }
            }
            handlers
        };
        // Get keys of successfuly calculated caches.
        // Return the full error if any worker fails.
        let calculated = {
            let dbgid = DbgId(format!("{}.update_caches", self.dbgid));
            let mut calculated = IndexSet::new();
            let mut errors = vec![];
            for (cache_key, workers) in handlers {
                for (id, handler) in workers {
                    match handler.join() {
                        Err(err) => {
                            log::error!("{} | Preparing thread='{}'..", dbgid, id);
                            errors.push(format!("  thread_id='{}', {:?}", id, err));
                        }
                        Ok(worker_res) => {
                            if let Err(err) = worker_res {
                                log::error!("{} | Calculating cache in thread='{}'..", dbgid, id);
                                errors.push(format!("  thread_id='{}', {:?}", id, err));
                            } else {
                                calculated.insert(cache_key);
                            }
                        }
                    }
                }
            }
            if !errors.is_empty() {
                return Err(StrErr(errors.join("\n")));
            }
            calculated
        };
        for (cache_key, cache) in &mut self.caches {
            if calculated.contains(cache_key) {
                cache.reload();
            }
        }
        Ok(())
    }
}

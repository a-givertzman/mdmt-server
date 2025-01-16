mod cache;
mod model_tree;
//
use cache::FloatingPositionCache;
use model_tree::ModelTree;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::path::Path;
///
/// Ship object being model.
pub struct Ship<A> {
    dbgid: DbgId,
    hull_key: String,
    model_tree: ModelTree<A>,
    fp_cache: FloatingPositionCache,
}
//
//
impl<A> Ship<A>
where
    A: Clone + Send + 'static,
{
    ///
    /// Creates a new instance of [Ship].
    pub fn new(
        parent: &DbgId,
        model_key: impl AsRef<str>,
        model_path: impl AsRef<Path>,
        fp_cache_path: impl AsRef<Path>,
    ) -> Self {
        let dbgid = DbgId::with_parent(parent, "Ship");
        Self {
            dbgid: dbgid.clone(),
            hull_key: model_key.as_ref().to_string(),
            model_tree: ModelTree::new(&dbgid, model_path),
            fp_cache: FloatingPositionCache::new(&dbgid, &fp_cache_path),
        }
    }
    ///
    /// Generates internal caches.
    ///
    /// The field `caches` contains required cache keys to build.
    /// Remain it empty to rebuild all the caches.
    ///
    /// # Errors
    /// Internally it creates worker threads while building.
    /// The result error is a collection of all failed worker errors joined by '\n'.
    pub fn build_cache(&self, caches: &[&Path]) -> Result<(), StrErr> {
        let dbgid = DbgId(format!("{}.build_cache", self.dbgid));
        let workers = {
            let mut workers = vec![];
            let build_all = caches.is_empty();
            for cache_path in caches {
                if build_all || self.fp_cache.same_path(cache_path) {
                    self.fp_cache
                        .builder(&dbgid, &self.hull_key, &self.model_tree)
                        .build()
                        .map(|worker| workers.push(worker))?;
                }
            }
            workers
        };
        let errors = {
            let mut errors = vec![];
            for (thread_id, handler) in workers.into_iter().flatten() {
                if let Err(err) = handler.join() {
                    log::error!("{} | Preparing thread='{}'..", dbgid, thread_id);
                    errors.push(format!("  thread_id='{}', {:?}", thread_id, err));
                }
            }
            errors
        };
        if errors.is_empty() {
            return Ok(());
        }
        Err(StrErr(errors.join("\n")))
    }
}

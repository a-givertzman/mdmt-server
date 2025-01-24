pub(super) mod floating_position_cache;
//
use crate::cache::Cache;
use sal_sync::services::{
    entity::error::str_err::StrErr, service::service_handles::ServiceHandles,
};
use std::sync::{atomic::AtomicBool, Arc};
///
/// A common trait for caches, which work with file system.
pub(super) trait LocalCache {
    ///
    /// Builds and stores the cache dataset.
    ///
    /// This method spawns a worker thread internally and returns its handler.
    /// Setting `exit` to _true_ at the caller side stops the worker.
    fn calculate(
        &self,
        exit: Arc<AtomicBool>,
    ) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr>;
    ///
    /// Returns approximated values based on given ones.
    fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>>;
    ///
    /// Reloads cached dataset.
    ///
    /// Typicaly, calling of this method should follow a call of [LocalCache::calculate].
    fn reload(&mut self);
}

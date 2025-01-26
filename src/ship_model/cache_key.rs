use strum_macros::EnumIter;
///
/// Cache keys of [ShipModel] caches.
#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum CacheKey {
    ///
    /// Points to [FloatingPositionCache].
    FloatingPostion,
}

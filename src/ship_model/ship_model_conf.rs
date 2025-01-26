use std::path::PathBuf;
///
/// [ShipModel] configuration.
///
/// It can be used to wrap configuration getting from an external source.
pub struct ShipModelConf {
    ///
    /// File containing model structure (e. g. in STEP format).
    pub model_path: PathBuf,
    ///
    /// [FloatingPositionCache] configuration.
    pub floating_position_cache_conf: FloatingPositionCacheConf,
}

///
/// [FloatingPositionCache] configuration.
pub struct FloatingPositionCacheConf {
    ///
    /// Used for reading cache and storing newly calculated ones.  !!! Is not specified in config !!!
    pub file_path: PathBuf,
    ///
    /// Models used for cache calculation.                         !!! Is not specified in config !!!
    pub model_keys: Vec<String>,
    ///
    /// Waterline initial position.
    pub waterline_position: [f64; 3],
    ///
    /// Angle in degree.
    pub heel_steps: Vec<f64>,
    ///
    /// Angle in degree.
    pub trim_steps: Vec<f64>,
    ///
    /// NOTE: needs to clarify.
    pub draught_steps: Vec<f64>,
}
//
//
impl Default for FloatingPositionCacheConf {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("floating_position_cache"),
            model_keys: vec![],
            waterline_position: [0.0; 3],
            heel_steps: vec![],
            trim_steps: vec![],
            draught_steps: vec![],
        }
    }
}

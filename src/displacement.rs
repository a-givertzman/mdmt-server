//! Водоизмещение судна

use crate::{math::Bound, Error};

/// Водоизмещение судна.
pub trait IDisplacement {
    /// Погруженный объем шпации.
    /// - bound: диапазон корпуса в длинну, для которого считается водоизмещение
    /// - draft: средняя осадка корпуса в диапазоне
    fn value(&self, bound: &Bound, draft_start: f64, draft_end: f64) -> Result<f64, Error>;
}

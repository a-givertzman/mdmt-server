//! Интерфейс для модели  
use crate::{error::Error, Bound, IDisplacement, LoadingType};
///
pub struct Model {}
///
impl Model {
    pub fn new() -> Self {
        Self {}
    }
}
///
impl IModel for Model {
    /// L.O.A
    fn loa(&self) -> f64 {
        todo!()
    }
    /// X midship from Fr0
    fn middle_x(&self) -> f64 {
        todo!()
    }
    /// Масса корпуса, ограниченная диапазоном по длинне
    /// * bound - ограничение по длинне
    /// * loading_type - тип нагрузки
    fn mass(&self, bound: &Bound, loading_type: LoadingType) -> f64 {
        todo!()
    }
}
///
impl IDisplacement for Model {
    /// Погруженный объем шпации.
    /// - bound: диапазон корпуса в длинну, для которого считается водоизмещение
    /// - draft: средняя осадка корпуса в диапазоне
    fn value(&self, bound: &Bound, draft_start: f64, draft_end: f64) -> Result<f64, Error> {
        todo!()
    }
}

#[doc(hidden)]
pub trait IModel {
    /// L.O.A
    fn loa(&self) -> f64;
    /// X midship from Fr0
    fn middle_x(&self) -> f64;
    /// Масса корпуса, ограниченная диапазоном по длинне
    /// * bound - ограничение по длинне
    /// * loading_type - тип нагрузки
    fn mass(&self, bound: &Bound, loading_type: LoadingType) -> f64;
}
// заглушка для тестирования
#[doc(hidden)]
pub struct FakeModel {
    loa: f64,
    middle_x: f64,
    mass: f64,
}
#[doc(hidden)]
#[allow(dead_code)]
impl FakeModel {
    pub fn new(loa: f64, middle_x: f64, mass: f64,) -> Self {
        Self {
            loa,
            middle_x,
            mass,
        }
    }
}
#[doc(hidden)]
impl IModel for FakeModel {
    ///
    fn loa(&self) -> f64 {
        self.loa
    }
    ///
    fn middle_x(&self) -> f64 {
        self.middle_x
    }
    ///
    fn mass(&self, _: &Bound, _: LoadingType) -> f64 {
        self.mass        
    }
}

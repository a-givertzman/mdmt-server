pub mod floating_position_cache_conf;
#[cfg(test)]
#[path = "../../../tests/models/ship_model/local_cache/floating_position_cache_test.rs"]
mod tests;
//
use super::{super::ModelTree, Cache, LocalCache};
use floating_position_cache_conf::FloatingPositionCacheConf;
use sal_3dlib::{
    gmath::vector::Vector,
    ops::transform::{Rotate, Translate},
    props::{Center, Volume},
    topology::shape::{
        compound::{AlgoMakerVolume, Compound, Solids},
        face::Face,
        vertex::Vertex,
        wire::{Polygon, Wire},
        Shape,
    },
};
use sal_sync::services::{
    entity::{dbg_id::DbgId, error::str_err::StrErr},
    service::service_handles::ServiceHandles,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
///
/// Pre-calculated cache for floating position algorithm.
///
/// See [FloatingPositionCacheConf] for more details about the fields.
pub(in super::super) struct FloatingPositionCache<A> {
    dbgid: DbgId,
    file_path: PathBuf,
    model_keys: Vec<String>,
    waterline_position: [f64; 3],
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Model representation used for cache calculation.
    model_tree: ModelTree<A>,
    ///
    /// Cache read from `self.file_path`.
    cache: Cache<f64>,
}
//
//
impl<A> FloatingPositionCache<A> {
    //
    //
    const KEY: &'static str = "floating_position_cache";
    ///
    /// Creates a new instance.
    /// - path - folder contains all cache files
    pub(in super::super) fn new(
        parent: &DbgId,
        model_tree: ModelTree<A>,
        path: impl AsRef<Path>,
        conf: FloatingPositionCacheConf,
    ) -> Self {
        let dbgid = DbgId::with_parent(parent, "FloatingPositionCache");
        let file_path = path.as_ref().join(Self::KEY);
        Self {
            model_tree,
            model_keys: vec![],
            heel_steps: conf.heel_steps,
            waterline_position: conf.waterline_position,
            trim_steps: conf.trim_steps,
            draught_steps: conf.draught_steps,
            cache: Cache::new(&dbgid, &file_path),
            file_path,
            dbgid,
        }
    }
    ///
    /// Creates a waterline object in 3D space centered at `self.waterline_position`.
    ///
    /// The result object is used for calculating cache algorithm (see [FloatingPositionCache::calculate]).
    fn create_waterline<T>(&self) -> Result<Face<T>, StrErr> {
        let dbgid = DbgId(format!("{}.create_waterline_model", self.dbgid));
        let [x, y, z] = self.waterline_position;
        // dynamic range could be built based on bounding box of target element behind self.model_keys,
        // but now reserve big enough offsets, which should work with most elements
        let dx = 1000.0;
        let dy = 1000.0;
        //
        match Wire::polygon(
            [
                Vertex::new([x + dx, y + dy, z]),
                Vertex::new([x - dx, y + dy, z]),
                Vertex::new([x - dx, y - dy, z]),
                Vertex::new([x + dx, y - dy, z]),
            ],
            true,
        ) {
            Ok(ref polygon) => Face::try_from(polygon).map_err(|why| {
                StrErr(format!(
                    "{} | Failed creating Face from *polygon*: {}",
                    dbgid, why
                ))
            }),
            Err(why) => Err(StrErr(format!(
                "{} | Failed creating *polygon* from Wire: {}",
                dbgid, why
            ))),
        }
    }
}
//
//
impl<A: Clone + Send + 'static> LocalCache for FloatingPositionCache<A> {
    ///
    /// See [CalculatedFloatingPositionCache] for details.
    fn calculate(
        &self,
        exit: Arc<AtomicBool>,
    ) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr> {
        CalculatedFloatingPositionCache::new(
            &self.dbgid,
            self.file_path.clone(),
            self.model_tree
                .iter()
                .filter_map(|(shape_key, shape)| {
                    self.model_keys.contains(shape_key).then_some(shape)
                })
                .cloned()
                .collect(),
            self.create_waterline()?,
            self.heel_steps.clone(),
            self.trim_steps.clone(),
            self.draught_steps.clone(),
            exit,
        )
        .build()
    }
    ///
    /// See [Cache::get] for details.
    fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>> {
        self.cache.get(approx_vals)
    }
    //
    //
    fn reload(&mut self) {
        self.cache = Cache::new(&self.dbgid, &self.file_path);
    }
}
///
/// Provides logic to calculate and store cache used by [FloatingPositionCache].
///
/// See [FloatingPositionCacheConf] for more details about the fields.
struct CalculatedFloatingPositionCache<A> {
    dbgid: DbgId,
    file_path: PathBuf,
    elements: Vec<Shape<A>>,
    waterline: Face<A>,
    heel_steps: Vec<f64>,
    trim_steps: Vec<f64>,
    draught_steps: Vec<f64>,
    ///
    /// Used to stop started worker thread.
    ///
    /// See [CalculatedFloatingPositionCache::calculate] for details.
    exit: Arc<AtomicBool>,
}
//
//
impl<A: Clone> CalculatedFloatingPositionCache<A> {
    ///
    /// Crates a new instance.
    #[allow(clippy::too_many_arguments)]
    fn new(
        parent: &DbgId,
        file_path: PathBuf,
        elements: Vec<Shape<A>>,
        waterline: Face<A>,
        heel_steps: Vec<f64>,
        trim_steps: Vec<f64>,
        draught_steps: Vec<f64>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "CalculatedFloatingPositionCache"),
            file_path,
            elements,
            waterline,
            heel_steps,
            trim_steps,
            draught_steps,
            exit,
        }
    }
    ///
    /// Creates and starts worker for [FloatingPositionCache::calculate].
    fn build(self) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr>
    where
        A: Send + 'static,
    {
        let dbgid = DbgId(format!("{}.build", self.dbgid));
        log::info!("{} | Starting...", dbgid);
        match thread::Builder::new()
            .name(self.dbgid.0.clone())
            .spawn(move || self.calculate())
        {
            Ok(handler) => {
                log::info!("{} | Starting - OK", dbgid);
                Ok(ServiceHandles::new(vec![(dbgid.0, handler)]))
            }
            Err(why) => {
                let err_msg = format!("{} | Starting - FAILED: {}", dbgid, why);
                log::warn!("{}", err_msg);
                Err(StrErr(err_msg))
            }
        }
    }
    ///
    /// Builds the cache and stores it into `self.file_path`.
    ///
    /// The caller can stop executing by setting `self.exit` to _true_.
    ///
    /// While calculating it iterates over `self.heel_steps`, `self.trim_steps`,
    /// and `self.draught_steps` to set a new position to cloned `self.waterline`.
    /// The cloned waterline is used to apply volume algorithm to `self.models`,
    /// to get, in order, _volume_ of all volumed parts placed under the waterline.
    /// At the end of each iteration, a line is written to the output file in format:
    /// "{heel_step} {trim_step} {draught_step} {volume}".
    fn calculate(self) -> Result<(), StrErr> {
        let dbgid = DbgId(format!("{}.calculate", self.dbgid));
        let out_f = &mut File::create(&self.file_path).map_err(|err| {
            StrErr(format!(
                "{} | Creating file='{}': {}",
                dbgid,
                self.file_path.display(),
                err
            ))
        })?;
        for &draught in &self.draught_steps {
            for &heel in &self.heel_steps {
                for &trim in &self.trim_steps {
                    // _true_ if the caller has requisted to exit.
                    // Note that in this case the file may be partially filled.
                    if self.exit.load(Ordering::SeqCst) {
                        log::warn!("{} | Interrupted: `exit` has got true", dbgid);
                        return Ok(());
                    }
                    // make a clone of origin waterline and transform it
                    // according to heel, trim, and draught values
                    let w_obj = &{
                        let mut obj = self.waterline.clone();
                        let origin = self.waterline.center();
                        let mut loc_y = Vector::unit_y();
                        if 0.0 != heel {
                            let heel_in_rad = heel.to_radians();
                            obj = obj.rotate(origin.clone(), Vector::unit_x(), heel_in_rad);
                            // once a rotation around oX happens, oY needs to get the rotation too,
                            // overwise oY remains global and doesn't match new `obj`'s transformation
                            loc_y = loc_y.rotate(Vector::unit_x(), heel_in_rad);
                        }
                        if 0.0 != trim {
                            obj = obj.rotate(origin, loc_y, trim.to_radians());
                        }
                        if 0.0 != draught {
                            obj = obj.translate(Vector::new(0.0, 0.0, -draught));
                        }
                        obj
                    };
                    self.elements
                        .iter()
                        .filter_map(|elmnt| {
                            // get compound as result of volume algorithm
                            // applied to waterline and each target element
                            // (taking into account its shape type)
                            Some(match elmnt {
                                Shape::Face(elmnt) => Compound::build([w_obj, elmnt], [], []),
                                Shape::Shell(elmnt) => Compound::build([w_obj], [elmnt], []),
                                Shape::Solid(elmnt) => Compound::build([w_obj], [], [elmnt]),
                                _ => return None,
                            })
                        })
                        .try_fold(0.0, |total_volume, build| {
                            build.map(|volumed| {
                                total_volume
                                    + volumed
                                        .solids()
                                        .into_iter()
                                        .map(|elmnt| {
                                            let [.., elmnt_z] = elmnt.center().point();
                                            let [.., waterline_z] = w_obj.center().point();
                                            // Only calculate volume if volumed element is below waterline.
                                            // Put 0.0 if it's not for consistent.
                                            (elmnt_z < waterline_z)
                                                .then(|| elmnt.volume())
                                                .unwrap_or_default()
                                        })
                                        .sum::<f64>()
                            })
                        })
                        .and_then(|volume| {
                            writeln!(out_f, "{} {} {} {}", heel, trim, draught, volume).map_err(
                                |err| {
                                    StrErr(format!(
                                        "{} | Writing to file='{}': {}",
                                        dbgid,
                                        self.file_path.display(),
                                        err
                                    ))
                                },
                            )
                        })?;
                }
            }
        }
        Ok(())
    }
}

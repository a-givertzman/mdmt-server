#[cfg(test)]
#[path = "../tests/model/cache.rs"]
mod tests;
//
use super::model_tree::ModelTree;
use crate::cache::Cache;
use indexmap::IndexMap;
use sal_3dlib::{
    gmath::Vector,
    ops::{
        transform::{Rotate, Translate},
        Polygon, Solidify,
    },
    props::{Center, Volume as VolumeProp},
    topology::{
        shape::{compound::Solids, Compound, Face, Solid, Vertex, Wire},
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
    ops::Deref,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
///
/// Pre-calculated dataset for floating position algorithm.
pub(super) struct FloatingPositionCache {
    dbgid: DbgId,
    path: PathBuf,
    inner: Cache<f64>,
}
//
//
impl FloatingPositionCache {
    ///
    /// Creates a new instance of [FloatingPositionCache].
    pub(super) fn new(parent: &DbgId, path: impl AsRef<Path>) -> Self {
        let dbgid = DbgId::with_parent(parent, "FloatingPositionCache");
        Self {
            dbgid: dbgid.clone(),
            path: path.as_ref().to_path_buf(),
            inner: Cache::new(&dbgid, &path),
        }
    }
    ///
    /// Check whether the instance path is the same as provided `path`.
    pub(super) fn same_path(&self, path: impl AsRef<Path>) -> bool {
        self.path == path.as_ref()
    }
    ///
    /// Creates a new instance of associated [builder].
    ///
    /// [builder]: FloatingPositionCacheBuilder
    pub(super) fn builder<A>(
        &self,
        dbgid: &DbgId,
        model_key: impl AsRef<str>,
        model_tree: &ModelTree<A>,
    ) -> FloatingPositionCacheBuilder<Option<A>>
    where
        A: Clone + Send + 'static,
    {
        FloatingPositionCacheBuilder::new(
            dbgid,
            &self.path,
            model_key,
            model_tree.deref().clone(),
            [vec![], vec![], vec![]],
            Arc::default(),
        )
    }
}
//
//
impl Deref for FloatingPositionCache {
    type Target = Cache<f64>;
    //
    //
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
///
/// Dataset builder for floating position [cache].
///
/// [cache]: FloatingPositionCache
pub(super) struct FloatingPositionCacheBuilder<A> {
    dbgid: DbgId,
    output_file: PathBuf,
    model_key: String,
    models: IndexMap<String, Shape<A>>,
    steps: [Vec<f64>; 3],
    exit: Arc<AtomicBool>,
}
//
//
impl<A> FloatingPositionCacheBuilder<A>
where
    A: Clone + Send + 'static,
{
    ///
    /// Crates a new instance.
    ///
    /// # Fields
    /// - `model`:
    ///   - `model`.0 - key string of target model,
    ///   - `model`.1 - model tree,
    /// - `steps`:
    ///   - `steps`\[0] - heel steps (in degrees),
    ///   - `steps`\[1] - trim steps (in degrees),
    ///   - `steps`\[2] - draught steps.
    pub(super) fn new(
        parent: &DbgId,
        output_file: impl AsRef<Path>,
        model_key: impl AsRef<str>,
        models: IndexMap<String, Shape<A>>,
        steps: [Vec<f64>; 3],
        exit: Arc<AtomicBool>,
    ) -> Self {
        let dbgid = DbgId::with_parent(parent, "FloatingPositionCacheBuilder");
        Self {
            dbgid: dbgid.clone(),
            output_file: output_file.as_ref().to_path_buf(),
            model_key: model_key.as_ref().to_string(),
            models,
            steps,
            exit,
        }
    }
    ///
    /// Builds the dataset and stores it into 'output_file'.
    ///
    /// This method spawns a worker thread internally and returns its handler as result.
    /// Setting `exit` to _true_ at the caller side stops the worker thread.
    pub(super) fn build(self) -> Result<ServiceHandles<Result<(), StrErr>>, StrErr> {
        let dbgid = DbgId(format!("{}.build", self.dbgid));
        log::info!("{} | Starting...", dbgid);
        match thread::Builder::new()
            .name(self.dbgid.0.clone())
            .spawn(move || self.build_inner())
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
    /// Actual implementation of [Self::build].
    ///
    /// The dataset is built by rotating the target model in 3 dimentions.
    /// Each dimention is paramenterized by values of according vector:
    /// - `steps`\[0] - heel angle of each step,
    /// - `steps`\[1] - trim angle of each step,
    /// - `steps`\[2] - draught, vertical moving.
    ///
    /// The target model by `model`.0 is a part of model tree stored in `model`.1.
    fn build_inner(self) -> Result<(), StrErr> {
        let dbgid = DbgId(format!("{}.build_inner", self.dbgid));
        let t_model = self.get_target_model()?;
        let w_model = self.create_waterline_model(t_model)?;
        let out_f = &mut File::create(&self.output_file).map_err(|err| {
            StrErr(format!(
                "{} | Creating file='{}': {}",
                dbgid,
                self.output_file.display(),
                err
            ))
        })?;
        for &draught in &self.steps[2] {
            for &heel in &self.steps[0] {
                for &trim in &self.steps[1] {
                    if self.exit.load(Ordering::SeqCst) {
                        log::warn!("{} | Interrupted: `exit` has got true", dbgid);
                        return Ok(());
                    }
                    let w_model = &{
                        let mut model = w_model.clone();
                        let mut loc_y = Vector::unit_y();
                        if 0.0 != heel {
                            let heel_in_rad = heel.to_radians();
                            model = model.rotated(w_model.center(), Vector::unit_x(), heel_in_rad);
                            // once a rotation around oX happens, oY needs to get the rotation too,
                            // overwise oY remains global and doesn't match new `model`'s transformation
                            loc_y = loc_y.rotated(Vector::unit_x(), heel_in_rad);
                        }
                        if 0.0 != trim {
                            model = model.rotated(w_model.center(), loc_y, trim.to_radians());
                        }
                        if 0.0 != draught {
                            model = model.translated(Vector::new(0.0, 0.0, -draught));
                        }
                        model
                    };
                    Compound::solidify([w_model], [], [t_model])
                        .map(|compound| {
                            compound
                                .solids()
                                .into_iter()
                                .map(|solid| {
                                    (solid.center().point()[2] < w_model.center().point()[2])
                                        .then(|| solid.volume())
                                        .unwrap_or_default()
                                })
                                .sum::<f64>()
                        })
                        .and_then(|volume| {
                            writeln!(out_f, "{} {} {} {}", heel, trim, draught, volume).map_err(
                                |err| {
                                    StrErr(format!(
                                        "{} | Writing to file='{}': {}",
                                        dbgid,
                                        self.output_file.display(),
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
    ///
    /// Extracts model from `self.model`.
    fn get_target_model(&self) -> Result<&Solid<A>, StrErr> {
        let dbgid = DbgId(format!("{}.get_target_model", self.dbgid));
        if let Shape::Solid(ref model) = self.models.get(&self.model_key).ok_or(StrErr(format!(
            "{} | Expected a value but got None by `model_key`='{}'",
            dbgid, self.model_key
        )))? {
            return Ok(model);
        }
        Err(StrErr(format!(
            "{} | Expected Solid by `model_key`='{}'",
            dbgid, self.model_key
        )))
    }
    ///
    /// Creates a waterline centered at `center`.
    fn create_waterline_model(
        &self,
        t_model: &impl Center<Output = Vertex<A>>,
    ) -> Result<Face<A>, StrErr> {
        let dbgid = DbgId(format!("{}.create_waterline_model", self.dbgid));
        let [x, y, z] = t_model.center().point();
        // dynamic range could be built based on bounding box of `t_model`,
        // but now reserve big enough offsets, which should work with most models
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
    ///
    /// Sends _exit_ signal to the worker thread.
    #[allow(unused)]
    pub fn exit(&self) {
        self.exit.store(true, Ordering::SeqCst);
    }
}

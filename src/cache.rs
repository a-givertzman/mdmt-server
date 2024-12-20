mod bound;
mod column;
mod table;
#[cfg(test)]
#[path = "./tests/cache.rs"]
mod tests;
//
use column::Column;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::fs::File;
use std::io::BufReader;
use std::sync::OnceLock;
use std::{
    io::BufRead,
    num::ParseFloatError,
    path::{Path, PathBuf},
    str::FromStr,
};
use table::Table;
//
type OwnedSet<T> = std::sync::Arc<[T]>;
type Float = f64;
///
/// Set of pre-loaded values.
pub(crate) struct Cache<T> {
    dbgid: DbgId,
    path: PathBuf,
    table: OnceLock<Result<Table<T>, StrErr>>,
}
//
//

impl<T> Cache<T> {
    ///
    /// Creates a new instance of [Cache].
    pub(crate) fn new(parent: &DbgId, path: impl AsRef<Path>) -> Self {
        Self {
            dbgid: DbgId::with_parent(parent, "Cache"),
            path: path.as_ref().to_owned(),
            table: OnceLock::new(),
        }
    }
}
//
//
impl<T: PartialOrd> Cache<T> {
    ///
    /// Creates an instance using `reader` as the source of values.
    ///
    /// # Panics
    /// Panic occurs if reader produces a non-comparable value (e. g. _NaN_).
    fn init(&self) -> Result<Table<T>, StrErr>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default,
    {
        let callee = "init";
        let file = File::open(&self.path).map_err(|err| {
            format!(
                "{}.{} | Failed reading file='{}': {}",
                self.dbgid,
                callee,
                self.path.display(),
                err
            )
        })?;
        let reader = BufReader::new(file);
        let mut vals = None;
        for (try_line, line_id) in reader.lines().zip(1..) {
            let line = try_line.map_err(|err| {
                format!(
                    "{}.{} | Failed reading line={}: {}",
                    self.dbgid, callee, line_id, err
                )
            })?;
            let ss = line.split_ascii_whitespace();
            let ss_len = ss.clone().count();
            let vals_mut = match vals.as_mut() {
                None => vals.insert(vec![vec![]; ss_len]),
                Some(vals) if vals.len() != ss_len => {
                    return Err(format!(
                        "{}.{} | Inconsistent dataset at line={}",
                        self.dbgid, callee, line_id
                    )
                    .into())
                }
                Some(vals) => vals,
            };
            for (i, s) in ss.enumerate() {
                let val = s.parse().map_err(|err| {
                    format!(
                        "{}.{} | Failed parsing value at line={}: {}",
                        self.dbgid, callee, line_id, err
                    )
                })?;
                vals_mut[i].push(val);
            }
        }
        let cols = vals
            .map(|vals| {
                let iter_over_cols = vals.into_iter().enumerate().map(|(id, vals)| {
                    let dbgid = DbgId::with_parent(&self.dbgid, &format!("Column_{}", id));
                    Column::new(dbgid, vals)
                });
                OwnedSet::from_iter(iter_over_cols)
            })
            .unwrap_or_default();
        Ok(Table::new(&self.dbgid, cols))
    }
}
//
//
impl Cache<Float> {
    ///
    /// Returns approximated values based on given ones.
    ///
    /// This is a safe method in terms of bounds: If `approx_vals` has more elements than [Cache] supports,
    /// this method returns `None`. In contrast, the empty vector returns if no value found.
    ///
    /// # Panics
    /// Panic occurs if `approx_vals` contains a non-comparable value (e. g. _NaN_).
    pub(crate) fn get(&self, approx_vals: &[Option<Float>]) -> Option<Vec<Vec<Float>>> {
        self.table
            .get_or_init(|| self.init())
            .as_ref()
            .unwrap_or_else(|err| {
                panic!(
                    "{}.{} | Failed initializing Table: {}",
                    self.dbgid, "get", err
                )
            })
            .get(approx_vals)
    }
}

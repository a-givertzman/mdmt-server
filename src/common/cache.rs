//!
//! Generic cache implementation.
//!
//! This implemetation can be used either directly or
//! be taken to create a more specific cache structure.
//
mod bound;
mod column;
mod table;
#[cfg(test)]
#[path = "../tests/common/cache_test.rs"]
mod tests;
//
use column::Column;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::ParseFloatError,
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};
use table::Table;
//
type OwnedSet<T> = std::sync::Arc<[T]>;
///
/// Cached dataset lazyly read from the file on the first access.
///
/// # Examples
/// ```
/// use sal_sync::services::entity::dbg_id::DbgId;
/// //
/// // only initializing, no file reading
/// let dbgid = DbgId("cache creator".to_owned());
/// let file_path = "/path/to/cache/file";
/// let cache = Cache::new(&dbgid, file_path);
/// // the first call causes reading file
/// let _ = cache.get(&[None, Some(1.0)]);
/// // the second call uses taken dataset
/// let _ = cache.get(&[Some(2.0)]);
/// ```
pub struct Cache<T> {
    dbgid: DbgId,
    path: PathBuf,
    table: OnceLock<Result<Table<T>, StrErr>>,
}
//
//
impl<T> Cache<T> {
    ///
    /// Creates a new instance.
    ///
    /// Note that this call doesn't read the file yet.
    /// The first access (see [Cache::get]) causes file reading.
    pub fn new(parent: &DbgId, path: impl AsRef<Path>) -> Self {
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
    /// Initializes Table reading `self.path` file.
    ///
    /// # Panics
    /// Panic occurs if the reader produces a non-comparable value (e. g. _NaN_).
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
impl Cache<f64> {
    ///
    /// Returns approximated values based on given set.
    ///
    /// This is a safe method in terms of bounds: If `approx_vals` has more elements than [Cache] supports,
    /// this method returns `None`. In contrast, the empty vector returns if no value found.
    ///
    /// # Panics
    /// This method panics if at least one of the statements is true:
    /// - `approx_vals` contains a non-comparable value (e. g. _NaN_),
    /// - reading file at `self.path` failed,
    /// - data of the `self.file` is inconsistent (parsing float error or missed data).
    ///
    /// # Examples
    /// ```
    /// fn explaination(cache: Cache<f64>) {
    ///     // get all rows of the file behind `cache`
    ///     let _ = cache.get(&[]);
    ///     // get approximated (or equal) rows, which values are calculated
    ///     // as the average of each columns between top and low bounds
    ///     // (the bounds are selected only for the first column):
    ///     // *cache file*
    ///     // |  ...     |
    ///     // |  0.0 ... | <-- top bound row
    ///     // | (0.5)    | <-- given value
    ///     // |  1.0 ... | <-- low bound row
    ///     // |  ...     |
    ///     // ------------
    ///     // ... - one or more values of type f64
    ///     let _ = cache.get(&[Some(0.5)]);
    ///     // similar to the the previous example,
    ///     // but the bounds are selected for the 2nd and 4th columns:
    ///     // *cache file*
    ///     // | *  ... ... ...  ... |
    ///     // | *  0.0  *  0.1  ... | <-- top bound row
    ///     // |   (0.1)   (0.2)     | <-- given values
    ///     // | *  1.0  *  0.5  ... | <-- low bound row
    ///     // | *  ... ... ...  ... |
    ///     // -----------------------
    ///     // * - any value of type f64
    ///     let _ = cache.get(&[None, Some(0.1), None, Some(0.2)]);
    /// }
    /// ```
    pub fn get(&self, approx_vals: &[Option<f64>]) -> Option<Vec<Vec<f64>>> {
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

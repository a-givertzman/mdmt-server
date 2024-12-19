mod bound;
mod column;
mod table;
//
use column::Column;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::{io::BufRead, num::ParseFloatError, str::FromStr};
use table::Table;
//
type OwnedSet<T> = std::sync::Arc<[T]>;
type Float = f64;
///
/// Set of pre-loaded values.
pub(crate) struct Cache<T> {
    dbgid: DbgId,
    table: Table<T>,
}
//
//
impl<T: PartialOrd> Cache<T> {
    ///
    /// Creates an instance using `reader` as the source of values.
    ///
    /// # Panics
    /// Panic occurs if reader produces a non-comparable value (e. g. _NaN_).
    pub(crate) fn from_reader_with_precision(
        dbgid: &DbgId,
        reader: impl BufRead,
    ) -> Result<Self, StrErr>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default,
    {
        let dbgid = DbgId::with_parent(dbgid, "Cache");
        let callee = "from_reader_with_precision";
        let mut vals = None;
        for (try_line, line_id) in reader.lines().zip(1..) {
            let line = try_line.map_err(|err| {
                format!(
                    "{}.{} | Failed reading line={}: {}",
                    dbgid, callee, line_id, err
                )
            })?;
            let ss = line.split_ascii_whitespace();
            let ss_len = ss.clone().count();
            let vals_mut = match vals.as_mut() {
                None => vals.insert(vec![vec![]; ss_len]),
                Some(vals) if vals.len() != ss_len => {
                    return Err(format!(
                        "{}.{} | Inconsistent dataset at line={}",
                        dbgid, callee, line_id
                    )
                    .into())
                }
                Some(vals) => vals,
            };
            for (i, s) in ss.enumerate() {
                let val = s.parse().map_err(|err| {
                    format!(
                        "{}.{} | Failed parsing value at line={}: {}",
                        dbgid, callee, line_id, err
                    )
                })?;
                vals_mut[i].push(val);
            }
        }
        let cols = vals
            .map(|vals| {
                let iter_over_cols = vals.into_iter().map(|vals| Column::new(vals));
                OwnedSet::from_iter(iter_over_cols)
            })
            .unwrap_or_default();
        let table = Table::new(&dbgid, cols);
        Ok(Self { dbgid, table })
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
        self.table.get(approx_vals)
    }
}

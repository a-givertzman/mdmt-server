mod bound;
mod column;
mod table;
//
use crate::error::StrErr;
use column::{ApproxOrd, Column};
use sal_sync::services::entity::dbg_id::DbgId;
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
impl<T: ApproxOrd> Cache<T> {
    ///
    /// Creates an instance with given `precision` using `reader` as the source of values.
    pub(crate) fn from_reader_with_precision(
        dbgid: &DbgId,
        reader: impl BufRead,
        precision: u8,
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
                    return Err(StrErr::from(format!(
                        "{}.{} | Inconsistent dataset at line={}",
                        dbgid, callee, line_id
                    )))
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
                let iter_over_cols = vals.into_iter().map(|vals| Column::new(vals, precision));
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
    pub(crate) fn get(&self, approx_vals: &[Option<Float>]) -> Option<Vec<Vec<Float>>> {
        self.table.get(approx_vals)
    }
}

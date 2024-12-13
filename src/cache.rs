mod bound;
mod column;
mod table;
//
use column::{ApproxOrd, Column};
use std::{
    io::{self, BufRead},
    num::ParseFloatError,
    str::FromStr,
};
use table::Table;
//
type OwnedSet<T> = std::sync::Arc<[T]>;
type Float = f64;
///
/// All errors [Cache] can fail with.
#[derive(Debug)]
pub(crate) enum CacheError<T: FromStr> {
    ///
    /// Failed reading from IO.
    Io(io::Error),
    ///
    /// Failed to convert value to target type `T`.
    ParseError(T::Err),
    ///
    /// Provided dataset is inconsistent at `line`.
    InconsistentDataset { line: usize },
}
//
//
impl<T: FromStr> From<io::Error> for CacheError<T> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
//
//
impl<T> From<ParseFloatError> for CacheError<T>
where
    T: FromStr<Err = ParseFloatError>,
{
    fn from(error: ParseFloatError) -> Self {
        Self::ParseError(error)
    }
}
///
/// Set of pre-loaded values.
#[derive(Default)]
pub(crate) struct Cache<T> {
    table: Table<T>,
}
//
//
impl<T: ApproxOrd> Cache<T> {
    ///
    /// Creates an instance with given `precision` using `reader` as the source of values.
    pub(crate) fn from_reader_with_precision(
        reader: impl BufRead,
        precision: u8,
    ) -> Result<Self, CacheError<T>>
    where
        T: FromStr<Err = ParseFloatError> + Clone + Default,
    {
        let mut vals = None;
        for (try_line, line_id) in reader.lines().zip(1..) {
            let line = try_line?;
            let ss = line.split_ascii_whitespace();
            let ss_len = ss.clone().count();
            let vals_mut = match vals.as_mut() {
                None => vals.insert(vec![vec![]; ss_len]),
                Some(vals) if vals.len() != ss_len => {
                    return Err(CacheError::InconsistentDataset { line: line_id })
                }
                Some(vals) => vals,
            };
            for (i, s) in ss.enumerate() {
                let val = s.parse()?;
                vals_mut[i].push(val);
            }
        }
        Ok(vals
            .map(|vals| {
                let iter_over_cols = vals.into_iter().map(|vals| Column::new(vals, precision));
                let columns = OwnedSet::from_iter(iter_over_cols);
                let table = Table::from(columns);
                Self { table }
            })
            .unwrap_or_default())
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

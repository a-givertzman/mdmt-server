#[cfg(test)]
#[path = "../tests/cache/table.rs"]
mod tests;
//
use crate::cache::{bound::Bound, column::Column, OwnedSet};

use super::Float;
///
/// Set of [Column]s.
#[derive(Default)]
pub(in crate::cache) struct Table<T> {
    columns: OwnedSet<Column<T>>,
}
//
//
impl<S, T> From<S> for Table<T>
where
    S: Into<OwnedSet<Column<T>>>,
{
    fn from(cols: S) -> Self {
        Self {
            columns: cols.into(),
        }
    }
}
//
//
impl Table<Float> {
    fn get_unchecked(&self, approx_vals: &[Option<Float>]) -> Vec<Vec<Float>> {
        let bounds = {
            let mut val_bounds = vec![];
            for (id, val) in approx_vals
                .iter()
                .enumerate()
                .filter_map(|(id, val)| val.as_ref().map(|val| (id, val)))
            {
                let bounds = self.columns[id].get_bounds(val);
                val_bounds.push(bounds);
            }
            loop {
                match (val_bounds.pop(), val_bounds.last_mut()) {
                    (None, _) => return vec![],
                    (Some(bounds), Some(last_bounds)) => {
                        //
                        // NOTE: switch between last_bounds and bounds may increase perf
                        *last_bounds = last_bounds
                            .iter()
                            .copied()
                            .flat_map(|last_bound| {
                                bounds
                                    .iter()
                                    .copied()
                                    .filter_map(|bound| match last_bound & bound {
                                        Bound::None => None,
                                        bound => Some(bound),
                                    })
                                    .collect::<Vec<Bound>>()
                            })
                            .collect();
                    }
                    (Some(mut last), None) => {
                        last.dedup();
                        break last;
                    }
                }
            }
        };
        bounds
            .into_iter()
            .flat_map(|bound| match bound {
                Bound::None => None,
                Bound::Single(row_id) => {
                    let mut vals = Vec::with_capacity(self.columns.len());
                    for col in self.columns.iter() {
                        let val = col[row_id];
                        vals.push(val);
                    }
                    Some(vals)
                }
                Bound::Range(start, end) => {
                    let mut vals = Vec::with_capacity(self.columns.len());
                    let len = end - start + 1;
                    for col in self.columns.iter() {
                        let sum = (start..=end).map(|row_id| col[row_id]).sum::<Float>();
                        vals.push(sum / len as Float);
                    }
                    Some(vals)
                }
            })
            .collect()
    }
}

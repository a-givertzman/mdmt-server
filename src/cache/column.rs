#[cfg(test)]
#[path = "../tests/cache/column.rs"]
mod tests;
//
use super::OwnedSet;
use crate::cache::bound::Bound;
use std::{cmp::Ordering, ops::Deref};
///
///
/// Analyzed dataset, column of [crate::cache::table::Table].
#[derive(Clone, Debug)]
pub(in crate::cache) struct Column<T> {
    inflections: OwnedSet<usize>,
    data: OwnedSet<T>,
}
//
//
impl<T: PartialOrd> Column<T> {
    ///
    /// Returns an instance analyzed with given precision.
    pub(in crate::cache) fn new<S>(values: S) -> Self
    where
        S: Into<OwnedSet<T>> + Deref<Target = [T]>,
    {
        Self {
            inflections: Self::get_inflections(&values),
            data: values.into(),
        }
    }
    ///
    /// Returns inflection point IDs based on given values and precision.
    fn get_inflections(values: &[T]) -> OwnedSet<usize> {
        use Ordering::*;
        //
        if values.is_empty() {
            return OwnedSet::from([]);
        }
        // inflection points
        let mut flex = vec![];
        // keep last flex direction
        let mut prev_dir = None;
        // iterate over [.., l_val, m_val, r_val, ..] values
        // with middle index:         ^ m_id
        for (win, m_id) in values.windows(3).zip(1..) {
            let l_val = &win[0];
            let m_val = &win[1];
            let r_val = &win[2];
            match (l_val.partial_cmp(m_val), m_val.partial_cmp(r_val)) {
                (Some(l_vs_m), Some(m_vs_r)) => match (l_vs_m, m_vs_r) {
                    // flex (p)oint: m_id __. . or   .
                    //                     . \    . .__ m_id
                    //                        p     ^p
                    (cur @ Less, Equal)
                    | (Equal, cur @ Less)
                    | (cur @ Greater, Equal)
                    | (Equal, cur @ Greater) => match prev_dir.as_mut() {
                        Some(prev) => {
                            if cur != *prev {
                                flex.push(m_id);
                                prev_dir = Some(cur);
                            }
                        }
                        None => prev_dir = Some(cur),
                    },
                    // local extremum
                    (Greater, cur @ Less) | (Less, cur @ Greater) => {
                        flex.push(m_id);
                        prev_dir = Some(cur);
                    }
                    _ => {}
                },
                _ => todo!("error handling of non-comparable value"),
            }
        }
        // Include the first, ..
        let mut ids = vec![0];
        if !flex.is_empty() {
            // .. middle, ..
            ids.extend(flex);
        }
        // ... and last indexes.
        ids.push(values.len() - 1);
        // remove duplicates
        ids.dedup();
        ids.into()
    }
    ///
    /// Returns bounds of given value within internal dataset.
    pub(in crate::cache) fn get_bounds(&self, val: &T) -> Vec<Bound> {
        use Ordering::*;
        // walk through all middle values
        let iter = self
            .inflections
            .windows(2)
            .filter(|win| {
                let first = &self.data[win[0]];
                let last = &self.data[win[1]];
                match (first.partial_cmp(val), val.partial_cmp(last)) {
                    (Some(first_vs_val), Some(val_vs_last)) => matches!(
                        (first_vs_val, val_vs_last),
                        (Less | Equal, Less | Equal) | (Greater | Equal, Greater | Equal)
                    ),
                    _ => todo!("error handling of non-comparable value"),
                }
            })
            .flat_map(|win| {
                Self::get_bounds_of_monotonic(&self.data[win[0]..=win[1]], val, win[0])
            });
        let mut bounds = Vec::from_iter(iter);
        bounds.dedup();
        bounds
    }
    ///
    /// Returns bounds of given value (`val`) placing in between elemnts of `vals`,
    /// where `offset` represents the actual start index of `vals`.
    /// Elements of `vals` are compared using [ApproxOrd] with given precision (`pr`).
    ///
    /// # Note
    /// If `vals` is not monotonic, the output is _meaningless_.
    fn get_bounds_of_monotonic(vals: &[T], val: &T, offset: usize) -> Vec<Bound> {
        let mut bounds = vec![];
        let dir = vals[0]
            .partial_cmp(&vals[vals.len() - 1])
            .expect("TODO: error handling of non-comparable value");
        let insert_id = vals.partition_point(|data_val| {
            let cmp_result = data_val
                .partial_cmp(val)
                .expect("TODO: error handling of non-comparable value");
            cmp_result == dir
        });
        if insert_id == 0 {
            bounds.push(Bound::Single(offset));
        } else if insert_id == vals.len() {
            bounds.push(Bound::Single(vals.len() - 1 + offset));
        } else if *val == vals[insert_id] {
            bounds.push(Bound::Single(insert_id + offset));
            match dir {
                Ordering::Less | Ordering::Greater => {
                    (1..)
                        .take_while(|i| {
                            vals.get(insert_id + i)
                                .is_some_and(|insert_val| insert_val == val)
                        })
                        .for_each(|i| {
                            bounds.push(Bound::Single(insert_id + i + offset));
                        });
                }
                _ => unreachable!(),
            }
        } else {
            let start = insert_id - 1 + offset;
            let end = insert_id + offset;
            bounds.push(Bound::Range(start, end));
        }
        bounds
    }
}
//
//
impl<T> Deref for Column<T> {
    type Target = [T];
    //
    //
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

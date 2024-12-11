#[cfg(test)]
#[path = "../tests/cache/column.rs"]
mod tests;
//
use super::OwnedSet;
use crate::cache::bound::Bound;
use std::{cmp::Ordering, ops::Deref};
///
/// Ordering methods (similar to [Ord]) with given approximation.
pub(in crate::cache) trait ApproxOrd<Rhs = Self> {
    ///
    /// Compare with precision.
    fn approx_cmp(&self, rhs: &Rhs, pr: u8) -> Ordering;
}
//
//
macro_rules! impl_approx_ord {
    ($($ty:ty),+) => {
        $(
            impl ApproxOrd<$ty> for $ty {
                fn approx_cmp(&self, rhs: &$ty, precision: u8) -> Ordering {
                    let base = 10 as $ty;
                    let pr = precision as i32;
                    let this = (self * base.powi(pr)).trunc();
                    let other = (rhs * base.powi(pr)).trunc();
                    this.total_cmp(&other)
                }
            }
        )+
    };
}
//
//
impl_approx_ord! { f32, f64 }
//
//
#[derive(Clone, PartialEq, Debug)]
enum DatasetType {
    Empty,
    Constant,
    NonIncreasing,
    NonDecreasing,
    RangeMonotonic(OwnedSet<usize>),
}
//
//
impl DatasetType {
    ///
    /// Defines a type based on values and precision.
    fn new<T: ApproxOrd>(values: &[T], precision: u8) -> Self {
        use DatasetType::*;
        use Ordering::*;
        //
        let len = values.len();
        match len {
            0 => Empty,
            1 => Constant,
            _ => {
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
                    match (
                        l_val.approx_cmp(m_val, precision),
                        m_val.approx_cmp(r_val, precision),
                    ) {
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
                    }
                }
                if !flex.is_empty() {
                    // Include the first, ..
                    let mut ids = vec![0];
                    // .. middle, ..
                    ids.extend(flex);
                    // ... and last indexes.
                    ids.push(values.len() - 1);
                    // remove duplicates
                    ids.dedup();
                    return RangeMonotonic(ids.into());
                }
                // compare the first and last element to get 'the shape'
                match values[0].approx_cmp(&values[len - 1], precision) {
                    Less => NonDecreasing,
                    Equal => Constant,
                    Greater => NonIncreasing,
                }
            }
        }
    }
}
///
/// Analyzed dataset, column of [Table] (see also [DatasetType]).
#[derive(Clone, Debug)]
pub(in crate::cache) struct Column<T> {
    ty: DatasetType,
    data: OwnedSet<T>,
    precision: u8,
}
//
//
impl<T> Column<T> {
    ///
    /// Returns an instance analyzed with given precision.
    pub(in crate::cache) fn new<S>(values: S, precision: u8) -> Self
    where
        S: Into<OwnedSet<T>> + Deref<Target = [T]>,
        T: ApproxOrd,
    {
        Self {
            ty: DatasetType::new(&values, precision),
            data: values.into(),
            precision,
        }
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
//
//
impl<T: ApproxOrd + std::fmt::Debug> Column<T> {
    ///
    /// Returns bounds of given value within internal dataset.
    pub(in crate::cache) fn get_bounds(&self, val: &T) -> Vec<Bound> {
        use DatasetType::*;
        use Ordering::*;
        //
        let mut bounds = vec![];
        match self.ty {
            Empty => { /* empty dataset, skip */ }
            Constant => bounds.push(Bound::Range(0, self.data.len() - 1)),
            RangeMonotonic(ref ids) => {
                // [to remove]
                // if `val` is lower than the first value,
                // take the value as the bound
                // if let Less | Equal = self.data[ids[0]].approx_cmp(val, self.precision) {
                //     bounds.push(Bound::Single(ids[0]));
                // }

                // walk through all middle values
                let iter = ids
                    .windows(2)
                    .filter(|win| {
                        let first = &self.data[win[0]];
                        let last = &self.data[win[1]];
                        let precision = self.precision;
                        let cmp_results = (
                            first.approx_cmp(val, precision),
                            val.approx_cmp(last, precision),
                        );
                        matches!(
                            cmp_results,
                            (Less | Equal, Less | Equal) | (Greater | Equal, Greater | Equal)
                        )
                    })
                    .flat_map(|win| {
                        Self::get_bounds_of_monotonic(
                            &self.data[win[0]..=win[1]],
                            val,
                            self.precision,
                            win[0],
                        )
                    });
                bounds.extend(iter);
                bounds.dedup();

                // [to remove]
                // let last = ids.len() - 1;
                // if let Equal = self.data[ids[last]].approx_cmp(val, self.precision) {
                //     bounds.push(Bound::Single(ids[last]));
                // }
                // if `val` is greater than the last value,
                // take the value as the bound
                // {
                //     let last = ids.len() - 1;
                //     if let Less | Equal = self.data[ids[last]].approx_cmp(val, self.precision) {
                //         bounds.push(Bound::Single(ids[last]));
                //     }
                // }
            }
            NonIncreasing | NonDecreasing => {
                let is_val_left_out = Greater == self.data[0].approx_cmp(val, self.precision);
                let is_val_right_out = {
                    let last = self.data.len() - 1;
                    Less == self.data[last].approx_cmp(val, self.precision)
                };
                if !is_val_left_out && !is_val_right_out {
                    bounds = Self::get_bounds_of_monotonic(
                        &self.data,
                        val,
                        self.precision,
                        0, /* no offset */
                    );
                }
            }
        }
        bounds
    }
    ///
    /// Returns bounds of given value (`val`) placing in between elemnts of `vals`,
    /// where `offset` represents the actual start index of `vals`.
    /// Elements of `vals` are compared using [ApproxOrd] with given precision (`pr`).
    ///
    /// # Note
    /// If `vals` is not monotonic, the output is _meaningless_.
    fn get_bounds_of_monotonic(vals: &[T], val: &T, pr: u8, offset: usize) -> Vec<Bound> {
        use Bound::*;
        //
        let mut bounds = vec![];
        let dir = vals[0].approx_cmp(&vals[vals.len() - 1], pr);
        let insert_id = vals.partition_point(|data_val| {
            let cmp_result = data_val.approx_cmp(val, pr);
            cmp_result == dir
        });
        // println!(" - vals={:?}, {}", vals, insert_id);
        if insert_id == 0 {
            bounds.push(Single(offset));
        } else if insert_id == vals.len() {
            bounds.push(Single(vals.len() - 1 + offset));
        } else if let Ordering::Equal = vals[insert_id].approx_cmp(val, pr) {
            bounds.push(Single(insert_id + offset));
            match dir {
                Ordering::Less | Ordering::Greater => {
                    (1..)
                        .take_while(|i| {
                            vals.get(insert_id + i).is_some_and(|insert_val| {
                                Ordering::Equal == insert_val.approx_cmp(val, pr)
                            })
                        })
                        .for_each(|i| {
                            bounds.push(Single(insert_id + i + offset));
                        });
                }
                _ => unreachable!(),
            }
        } else {
            // [to remove]
            // end of range
            // let end = (insert_id..)
            //     .take_while(|&id| {
            //         id < vals.len() && {
            //             let cmp = vals[insert_id].approx_cmp(&vals[id], pr);
            //             matches!(cmp, Ordering::Equal,)
            //         }
            //     })
            //     .last()
            //     .unwrap_or(insert_id);

            let start = insert_id - 1 + offset;
            let end = insert_id + offset;
            bounds.push(Range(start, end));
        }
        bounds
    }
}

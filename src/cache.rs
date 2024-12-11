#![allow(unused)]
//
use std::{
    cmp::Ordering,
    ops::{BitAnd, Deref},
    sync::Arc,
};
//
type Float = f64;
type OwnedSet<T> = Arc<[T]>;
///
/// Ordering methods (similar to [Ord]) with given approximation.
trait ApproxOrd<Rhs = Self> {
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
struct Column<T> {
    ty: DatasetType,
    data: OwnedSet<T>,
    precision: u8,
}
//
//
impl<T> Column<T> {
    ///
    /// Returns an instance analyzed with given precision.
    fn new<S>(values: S, precision: u8) -> Self
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
///
/// Represents bound(s) of element within the collection.
#[derive(PartialEq, Clone, Copy, Debug)]
enum Bound {
    ///
    /// No bound.
    None,
    ///
    /// Index of the match.
    Single(usize),
    ///
    /// Indexes of either two nearest neighbors,
    /// or the start and end of exact match range.
    Range(usize, usize),
}
//
//
impl BitAnd for Bound {
    type Output = Self;
    //
    //
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Bound::Single(this), Bound::Single(other)) if this == other => Bound::Single(this),
            (Bound::Single(id), Bound::Range(start, end))
            | (Bound::Range(start, end), Bound::Single(id))
                if (start..=end).contains(&id) =>
            {
                Bound::Single(id)
            }
            (Bound::Range(mut start1, mut end1), Bound::Range(mut start2, mut end2)) => {
                if start2 < start1 {
                    std::mem::swap(&mut start1, &mut start2);
                    std::mem::swap(&mut end1, &mut end2);
                }
                if end1 < start2 {
                    return Bound::None;
                }
                let start = start1.max(start2);
                let end = end1.min(end2);
                match start == end {
                    true => Bound::Single(start),
                    _ => Bound::Range(start, end),
                }
            }
            _ => Bound::None,
        }
    }
}
//
//
impl<T: ApproxOrd + std::fmt::Debug> Column<T> {
    ///
    /// Returns bounds of given value within internal dataset.
    fn get_bounds(&self, val: &T) -> Vec<Bound> {
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
    //
    // if given values are not monotonic, the output is meaningless
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
            // [to remote]
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
///
/// Set of [Column]s.
pub struct Table<T> {
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
    pub fn get_unchecked(&self, vals: &[Option<Float>]) -> Vec<Vec<Float>> {
        let bounds = {
            let mut val_bounds = vec![];
            for (id, val) in vals
                .iter()
                .enumerate()
                .filter_map(|(id, val)| val.as_ref().map(|val| (id, val)))
            {
                let bounds = self.columns[id].get_bounds(val);
                val_bounds.push(bounds);
            }
            // dbg!(&val_bounds);
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
        // dbg!(&bounds);
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
//
//
#[cfg(test)]
mod tests {
    use super::*;
    //
    //
    #[test]
    fn dataset_types() {
        use DatasetType::*;
        //
        #[rustfmt::skip]
        let test_data = [
            // 0
            (vec![], 0, Empty),
            (vec![1.], 1, Constant),
            (vec![2., 2.], 1, Constant),
            (vec![0., 1., 2.], 1, NonDecreasing),
            (vec![0., 1., 2., 2.], 1, NonDecreasing),
            (vec![0., 1., 1., 2.], 1, NonDecreasing),
            (vec![2., 1., 0.], 1, NonIncreasing),
            (vec![2., 2., 1., 0.], 1, NonIncreasing),
            (vec![2., 1., 0., 0.], 1, NonIncreasing),
            (vec![4., 4., 1., 4., 4.], 1, RangeMonotonic(vec![0, 2, 4].into())),
            // 10
            (vec![0., 0., 0., 0., 1.], 1, NonDecreasing),
            (vec![1., 0., 0., 0., 0.], 1, NonIncreasing), 
            (vec![4., 0., 0., 0., 4.], 1, RangeMonotonic(vec![0, 3, 4].into())),
            (vec![4., 0., 0., 0., 4., 0.], 1, RangeMonotonic(vec![0, 3, 4, 5].into())),
            (vec![4., 4., -2., 4., 4.], 1, RangeMonotonic(vec![0, 2, 4].into())),
            (vec![0., 0., 5., 0., 0., 5.], 1, RangeMonotonic(vec![0, 2, 4, 5].into())),
            (vec![0., 0., 7., 0., 0., 7., 7., 7.], 1, RangeMonotonic(vec![0, 2, 4, 7].into())),
            (vec![6., 5., 4., 3., 4., 5., 6.], 1, RangeMonotonic(vec![0, 3, 6].into())),
            (vec![0., 1., 2., 3., 2., 1., 0., 1., 2., 3., 2., 1., 0.], 1, RangeMonotonic(vec![0, 3, 6, 9, 12].into())),
            (vec![0., 0., 2., 2., 3., 3., 2., 2., 1., 1., 0., 0., 1., 0., 0.], 1, RangeMonotonic(vec![0, 5, 11, 12, 14].into())),
            // 20
            // NOTE: zig-zag shape like '/\/\/' gives values one-by-one
            // need to handle that somehow?
            (vec![-1., 1., -0.5, 0.5, 0.], 1, RangeMonotonic(vec![0, 1, 2, 3, 4].into())),
        ];
        for (step, (values, precision, result)) in test_data.into_iter().enumerate() {
            let actual = DatasetType::new(&values, precision);
            assert_eq!(actual, result, "step={} values={:?}", step, values);
        }
    }
    //
    //
    #[test]
    fn approx_cmp_f64() {
        let test_data_eq = [(0.1 + 0.2, 0.3, 16), (1e-10 + 1e-10 * 0.1, 1e-10, 10)];
        for (this, other, precision) in test_data_eq {
            assert_eq!(Ordering::Equal, this.approx_cmp(&other, precision));
        }
        //
        let test_data_ne = [(0.1 + 0.2, 0.3, 17), (1e-10 + 1e-10 * 0.1, 1e-10, 11)];
        for (this, other, precision) in test_data_ne {
            assert_ne!(Ordering::Equal, this.approx_cmp(&other, precision));
        }
    }
    //
    //
    #[test]
    fn monotonic_shape_bounds() {
        use Bound::*;
        //                0   1   2   3   4   5   6   7    8    9   10   11
        let values = vec![0., 1., 2., 3., 2., 1., 0., 0., -1., -1., 10., 9.];
        let precsion = 4;
        let datatype = DatasetType::new(&values, precsion);
        println!("datatype={:?}", datatype);
        let column = Column::new(values, precsion);
        let test_data = [
            // 0
            (3.5, vec![Range(9, 10)]),
            (0.0, vec![Single(0), Single(6), Single(7), Range(9, 10)]),
            (0.5, vec![Range(0, 1), Range(5, 6), Range(9, 10)]),
            (1.0, vec![Single(1), Single(5), Range(9, 10)]),
            (1.5, vec![Range(1, 2), Range(4, 5), Range(9, 10)]),
            (2.5, vec![Range(2, 3), Range(3, 4), Range(9, 10)]),
            (3.0, vec![Single(3), Range(9, 10)]),
            // NOTE: take while left/right is same?
            // e.g. (7, 8) -> (7, 9)
            //     (9, 10) -> (8, 10)
            (-0.1, vec![Range(7, 8), Range(9, 10)]),
            (-1.0, vec![Single(8), Single(9)]),
            (-1.1, vec![]),
            // 10
            (10.0, vec![Single(10)]),
            (9.5, vec![Range(9, 10), Range(10, 11)]),
            (8.5, vec![Range(9, 10)]),
        ];
        for (step, (value, target)) in test_data.into_iter().enumerate() {
            let result = column.get_bounds(&value);
            assert_eq!(
                result, target,
                "step={} target={:?}, result={:?}",
                step, target, result
            );
        }
    }
    //
    //
    #[test]
    fn non_descresing_shape_bounds() {
        use Bound::*;
        //                0   1   2   3   4   5   6   7
        let values = vec![0., 1., 1., 1., 2., 2., 3., 4.];
        let precsion = 4;
        let datatype = DatasetType::new(&values, precsion);
        println!("datatype={:?}", datatype);
        let column = Column::new(values, precsion);
        let test_data = [
            // 0
            (-1.0, vec![]),
            (0.0, vec![Single(0)]),
            (0.5, vec![Range(0, 1)]),
            (1.0, vec![Single(1), Single(2), Single(3)]),
            (1.5, vec![Range(3, 4)]),
            (2.0, vec![Single(4), Single(5)]),
            (5.0, vec![]),
        ];
        for (step, (value, target)) in test_data.into_iter().enumerate() {
            let result = column.get_bounds(&value);
            assert_eq!(
                result, target,
                "step={} target={:?}, result={:?}",
                step, target, result
            );
        }
    }
}

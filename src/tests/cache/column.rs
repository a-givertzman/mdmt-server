use super::*;
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

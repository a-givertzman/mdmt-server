use super::*;
//
#[test]
fn bitand_test() {
    let test_data = [
        // 0
        (Bound::None, Bound::None, Bound::None),
        (Bound::None, Bound::Single(0), Bound::None),
        (Bound::None, Bound::Range(0, 1), Bound::None),
        (Bound::Single(0), Bound::None, Bound::None),
        (Bound::Single(0), Bound::Single(0), Bound::Single(0)),
        // 5
        (Bound::Single(0), Bound::Single(1), Bound::None),
        (Bound::Single(0), Bound::Range(0, 1), Bound::Single(0)),
        (Bound::Single(1), Bound::Range(0, 1), Bound::Single(1)),
        (Bound::Single(2), Bound::Range(0, 1), Bound::None),
        (Bound::Single(1), Bound::Range(0, 2), Bound::Single(1)),
        // 10
        (Bound::Range(0, 1), Bound::None, Bound::None),
        (Bound::Range(0, 1), Bound::Single(0), Bound::Single(0)),
        (Bound::Range(0, 1), Bound::Single(1), Bound::Single(1)),
        (Bound::Range(0, 1), Bound::Single(2), Bound::None),
        (Bound::Range(0, 2), Bound::Single(1), Bound::Single(1)),
        // 15
        (Bound::Range(0, 1), Bound::Range(2, 3), Bound::None),
        (Bound::Range(2, 3), Bound::Range(0, 1), Bound::None),
        (Bound::Range(0, 1), Bound::Range(1, 2), Bound::Single(1)),
        (Bound::Range(1, 2), Bound::Range(0, 1), Bound::Single(1)),
        (Bound::Range(0, 2), Bound::Range(0, 1), Bound::Range(0, 1)),
        // 20
        (Bound::Range(0, 1), Bound::Range(0, 2), Bound::Range(0, 1)),
        (Bound::Range(0, 2), Bound::Range(1, 3), Bound::Range(1, 2)),
        (Bound::Range(1, 3), Bound::Range(0, 2), Bound::Range(1, 2)),
        (Bound::Range(0, 3), Bound::Range(1, 2), Bound::Range(1, 2)),
        (Bound::Range(1, 2), Bound::Range(0, 3), Bound::Range(1, 2)),
    ];
    for (step, (lhs, rhs, target)) in test_data.into_iter().enumerate() {
        let result = lhs & rhs;
        assert_eq!(
            result, target,
            "step={} lhs={:?}, rhs={:?}, target={:?}, result={:?}",
            step, lhs, rhs, target, result
        );
    }
}

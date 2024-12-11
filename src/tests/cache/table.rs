use super::*;
//
#[test]
fn get_value_from_table() {
    let precision = 4;
    let matrix = [
        // 0    1    2     3
        [0.0, 0.0, 0.0, 10.0], // 0
        [1.0, 0.0, 0.0, 20.0], // 1
        [0.0, 1.0, 0.0, 11.0], // 2
        [0.0, 0.0, 1.0, 10.1], // 3
        [1.0, 1.0, 0.0, 21.0], // 4
        [1.0, 1.0, 1.0, 21.1], // 5
        [0.0, 1.0, 1.0, 11.1], // 6
        [1.0, 0.0, 1.0, 20.1], // 7
    ];
    let mut columns = vec![];
    for col_id in 0..matrix[0].len() {
        let mut values = vec![];
        (0..matrix.len()).for_each(|row_id| {
            let var_name = matrix[row_id][col_id];
            values.push(var_name);
        });
        let column = Column::new(values, precision);
        // dbg!(&column);
        columns.push(column);
    }
    let table = Table::from(columns);
    #[rustfmt::skip]
         let test_data = [
             // 0
             ([Some(0.0), Some(1.0), Some(1.0)].as_slice(),   vec![vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), Some(1.0), Some(1.0), None],       vec![vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), Some(1.0), Some(1.0), Some(11.1)], vec![vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), Some(1.0), None, Some(11.1)],      vec![vec![0.0, 1.0, 0.0, 11.0], vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), None, Some(1.0), Some(11.1)],      vec![vec![0.0, 0.0, 1.0, 10.1], vec![0.0, 1.0, 1.0, 11.1]],),
             // 5
             (&[None, Some(1.0), Some(1.0), Some(11.1)], vec![vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), Some(1.0)],                   vec![vec![0.0, 1.0, 0.0, 11.0], vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), None, Some(1.0)],             vec![vec![0.0, 0.0, 1.0, 10.1], vec![0.0, 1.0, 1.0, 11.1]],),
             (&[Some(0.0), None, None, Some(11.0)],      vec![
                                                             vec![0.0, 0.0, 0.0, 10.0],
                                                             vec![0.0, 1.0, 0.0, 11.0],
                                                             vec![0.0, 0.0, 1.0, 10.1],
                                                         ],
             ),
             (&[None, Some(0.0), None, Some(10.1)],      vec![vec![0.0, 0.0, 0.0, 10.0], vec![0.0, 0.0, 1.0, 10.1]],),
             // 10
             (&[None, None, Some(0.0), Some(21.0)], vec![vec![1.0, 1.0, 0.0, 21.0]],),
             (&[Some(0.0)],                         vec![
                                                        vec![0.0, 0.0, 0.0, 10.0],
                                                        vec![0.0, 1.0, 0.0, 11.0],
                                                        vec![0.0, 0.0, 1.0, 10.1],
                                                        vec![0.0, 1.0, 1.0, 11.1],
                                                    ]
             ),
             (&[None, Some(0.0)],                   vec![
                                                        vec![0.0, 0.0, 0.0, 10.0],
                                                        vec![0.0, 0.0, 1.0, 10.1],
                                                        vec![1.0, 0.0, 1.0, 20.1],
                                                    ]
             ),
             (&[None, None, Some(0.0)],             vec![
                                                        vec![0.0, 0.0, 0.0, 10.0],
                                                        vec![1.0, 1.0, 0.0, 21.0],
                                                    ]
             ),
             (&[None, None, None, Some(21.0)],      vec![
                                                        vec![1.0, 1.0, 0.0, 21.0],
                                                        vec![0.5, 1.0, 1.0, 16.1]
                                                    ]
             ,),
             // 15 : cannot do that because it may change after left-right fix
             // (&[Some(0.5)], vec![
             //                    // vec![0.5, 0.0, 0.0, 15.0],
             //                    // vec![0.5, 0.5, 0.0, 15.5],
             //                    // vec![0.0, 0.0, 0.1, 10.1],
             //                ],
             // ),
         ];
    for (step, (value, target)) in test_data.into_iter().enumerate() {
        // if step != 15 {
        //     continue;
        // }
        let result = table.get_unchecked(value);
        assert_eq!(
            target, result,
            "step={} target={:?}, result={:?}",
            step, target, result
        );
    }
}

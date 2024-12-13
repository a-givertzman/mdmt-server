use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use std::{fs::File, io::BufReader, sync::Once, time::Duration};
use testing::stuff::max_test_duration::TestDuration;

use crate::cache::{Cache, CacheError};
//
//
static INIT: Once = Once::new();
///
/// Once called initialisation.
fn init_once() {
    //
    // Implement your initialisation code to be called only once for current test file.
    INIT.call_once(|| {})
}
///
/// Returns:
///  - ...
#[allow(clippy::unused_unit)]
fn init_each() -> () {}
///
/// Test successfull creating of [Cache] instance based on file reader.
#[test]
fn from_reader_with_precision_ok() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg_id = "from_reader_with_precision_ok";
    log::debug!("\n{}", dbg_id);
    let test_duration = TestDuration::new(dbg_id, Duration::from_secs(1));
    test_duration.run().unwrap();
    // init
    //
    let path = "src/tests/cache/tempdir/table-ok";
    let file = File::open(path).expect("file exists");
    let reader = BufReader::new(file);
    let precision = 1;
    //
    ////
    #[rustfmt::skip]
    let test_data = [
        ([Some(0.0), Some(0.0), Some(0.0), Some(10.0)], Some(vec![vec![0.0, 0.0, 0.0, 10.0]])),
        ([Some(2.1), Some(0.1), Some(0.1), Some(20.1)], Some(vec![vec![2.1, 0.1, 0.1, 20.1]])),
        ([Some(3.2), Some(1.2), Some(0.2), Some(30.2)], Some(vec![vec![3.2, 1.2, 0.2, 30.2]])),
        ([Some(4.3), Some(0.3), Some(1.3), Some(40.3)], Some(vec![vec![4.3, 0.3, 1.3, 40.3]])),
        ([Some(5.4), Some(2.4), Some(2.4), Some(50.4)], Some(vec![vec![5.4, 2.4, 2.4, 50.4]])),
        ([Some(0.5), Some(3.5), Some(0.5), Some(60.5)], Some(vec![vec![0.5, 3.5, 0.5, 60.5]])),
        ([Some(0.6), Some(4.6), Some(3.6), Some(70.6)], Some(vec![vec![0.6, 4.6, 3.6, 70.6]])),
        ([Some(0.7), Some(0.7), Some(4.7), Some(80.7)], Some(vec![vec![0.7, 0.7, 4.7, 80.7]])),
    ];
    let cache = Cache::from_reader_with_precision(reader, precision).expect("created");
    for (step, (vals, target)) in test_data.into_iter().enumerate() {
        let result = cache.get(&vals);
        println!(
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
        assert_eq!(
            target, result,
            "step={} vals={:?} target={:?} result={:?}",
            step, vals, target, result
        );
    }
    test_duration.exit();
}
//
//
#[test]
fn from_reader_with_precision_inconsistent() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbg_id = "from_reader_with_precision_inconsistent";
    log::debug!("\n{}", dbg_id);
    let test_duration = TestDuration::new(dbg_id, Duration::from_secs(1));
    test_duration.run().unwrap();
    let test_data = [
        ("src/tests/cache/tempdir/table-inc-row", 6),
        ("src/tests/cache/tempdir/table-inc-col", 8),
    ];
    for (path, target) in test_data {
        let file = File::open(path).expect("file exists");
        let reader = BufReader::new(file);
        let precision = 1;
        let result = Cache::<f64>::from_reader_with_precision(reader, precision);
        match result {
            Ok(_) => panic!("error expected"),
            Err(error) => {
                assert!(
                    matches!(error, CacheError::InconsistentDataset { line: result } if target == result),
                    "path={} result={:?} target={:?}",
                    path,
                    error,
                    CacheError::<f64>::InconsistentDataset { line: target }
                );
            }
        }
    }
    test_duration.exit();
}

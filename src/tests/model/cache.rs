use crate::model::cache::FloatingPositionCacheBuilder;
use debugging::session::debug_session::{Backtrace, DebugSession, LogLevel};
use indexmap::IndexMap;
use sal_3dlib::fs::Reader;
use sal_sync::services::entity::{dbg_id::DbgId, error::str_err::StrErr};
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    sync::{atomic::AtomicBool, Arc, Once},
    time::Duration,
};
use testing::stuff::max_test_duration::TestDuration;
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
/// Test creating dataset for Floating postion cache.
///
/// # Notes
/// During the test a file called `fpc_result` is created in ./tmpdir/.
/// At the end of the test it tries (safely) remove it.
/// Pay attention on loggin info (WARN level) to catch it fails cleaning up.
#[test]
fn build_floating_position_cache() {
    DebugSession::init(LogLevel::Info, Backtrace::Short);
    init_once();
    init_each();
    let dbgid = DbgId("test cache Builder/build_floating_position_cache".to_string());
    log::debug!("\n{}", dbgid);
    let test_duration = TestDuration::new(&dbgid, Duration::from_secs(300));
    test_duration.run().unwrap();
    let model_path = "src/tests/model/assets/cube_1_1_1.step";
    let target_path = "src/tests/model/assets/fpc_target";
    let result_path = "src/tests/model/tmpdir/fpc_result";
    // read STEP file to get model tree
    // and turn it into map
    let models = {
        let models = Reader::read_step(model_path)
            .unwrap_or_else(|err| panic!("Failed reading STEP file='{}': {}", model_path, err))
            .into_vec::<()>()
            .unwrap_or_else(|err| panic!("Failed getting vec from *tree*: {}", err));
        IndexMap::from_iter(models)
    };
    let model_key = "/cube_1_1_1_centered";
    assert!(models.contains_key(model_key));
    // build floating position cache
    let handlers = FloatingPositionCacheBuilder::new(
        &dbgid,
        result_path,
        model_key,
        models,
        [
            // heel
            (-10..=10).step_by(5).map(|n| n as f64).collect(),
            // trim
            (-10..=10).step_by(5).map(|n| n as f64).collect(),
            // draught
            vec![0.0, 0.25],
        ],
        Arc::new(AtomicBool::new(false)),
    )
    .build()
    .unwrap_or_else(|err| panic!("Failed creating *handlers*: {}", err));
    let mut errors = vec![];
    for (_, handler) in handlers {
        match handler.join() {
            Err(why) => {
                let err_msg = format!("Failed preparing thread: {:?}", why);
                errors.push(StrErr(err_msg));
            }
            Ok(res) => {
                if let Err(why) = res {
                    let err_msg = format!("Failed executing thread: {:?}", why);
                    errors.push(StrErr(err_msg));
                }
            }
        }
    }
    assert!(errors.is_empty(), "*errors*: {:?}", errors);
    // read target file
    let mut target_reader = {
        let target_file = File::open(target_path)
            .unwrap_or_else(|err| panic!("Failed opening target file='{}': {}", target_path, err));
        BufReader::new(target_file)
    };
    // read result file
    let mut result_reader = {
        let result_file = File::open(result_path)
            .unwrap_or_else(|err| panic!("Failed opening result file='{}': {}", result_path, err));
        BufReader::new(result_file)
    };
    // check files line-by-line
    for ((try_target_line, try_result_line), line_id) in target_reader
        .by_ref()
        .lines()
        .zip(result_reader.by_ref().lines())
        .zip(1..)
    {
        let target = try_target_line
            .unwrap_or_else(|err| panic!("line={} | Failed getting target line: {}", line_id, err));
        let result = try_result_line
            .unwrap_or_else(|err| panic!("line={} | Failed getting result line: {}", line_id, err));
        assert_eq!(
            target, result,
            "line={} target='{}' result='{}'",
            line_id, target, result
        );
    }
    // check remaining lines in both files
    let remaining_target_lines = target_reader.lines().count();
    assert_eq!(
        remaining_target_lines, 0,
        "*result_file*.lines.count < *target_file*.lines.count"
    );
    assert_eq!(
        0,
        result_reader.lines().count(),
        "*result_file*.lines.count > *target_file*.lines.count"
    );
    // clean up
    if let Err(why) = fs::remove_file(result_path) {
        log::warn!(
            "Clean up (optional) | Failed removing result file='{}': {}",
            result_path,
            why
        );
    }
    test_duration.exit();
}

mod cmdline;
mod epds;

use cmdline::CmdLine;
use epds::LARGE_TEST_EPDS;
use librustic::{
    board::Board,
    misc::perft,
    movegen::MoveGenerator,
    search::defs::{PerftData, TT},
};
use std::time::Instant;

const SEMI_COLON: char = ';';
const SPACE: char = ' ';

struct TestResult;
impl TestResult {
    pub const ERR_NONE: usize = 0;
    pub const ERR_FEN: usize = 1;
    pub const ERR_DEPTH: usize = 2;
    pub const ERR_EXPECT: usize = 3;
    pub const ERR_FAIL: usize = 4;
}

const TEST_RESULTS: [&str; 5] = [
    "No errors. Test completed successfully.",
    "Errors in parsing the FEN-string.",
    "Errors parsing depth from test data.",
    "Errors parsing expected leaf nodes from test data.",
    "Failure: Found leaf nodes not equal to expected value.",
];

// This private function is the one actually running tests.
// This can be the entire suite, or a single test.
pub fn main() {
    // TODO: Write About

    let cmdline = CmdLine::new();
    let tt_size = cmdline.hash();
    let number_of_tests = LARGE_TEST_EPDS.len();
    let move_generator = MoveGenerator::new();
    let mut board: Board = Board::new();
    let mut result: usize = TestResult::ERR_NONE;
    let mut transposition = TT::<PerftData>::new(tt_size);
    let tt_enabled = tt_size > 0;

    // Run all the tests.
    let mut test_nr = 0;
    while (test_nr < number_of_tests) && (result == 0) {
        // Split the test's data string into multiple parts.
        let test_data: Vec<&str> = LARGE_TEST_EPDS[test_nr].split(SEMI_COLON).collect();
        let fen = test_data[0].trim();

        // Set up the position according to the provided FEN-string.
        let setup_result = board.fen_setup(Some(fen));
        println!("Test {} from {}", test_nr + 1, number_of_tests);
        println!("FEN: {fen}");

        // If setup ok, then print position. Else, print error and continue to the next test.
        match setup_result {
            Ok(()) => println!("{board}"),
            Err(_) => result = TestResult::ERR_FEN,
        };

        // Run all the parts of a test.
        let mut index: usize = 1;
        while index < test_data.len() && (result == 0) {
            // Data index 0 contains the FEN-string, so skip this and
            // start at index 1 to find the expected leaf nodes per depth.

            // Split "D1 20" into a vector containing "D1" (depth) and "20" (leaf nodes)
            let depth_ln: Vec<&str> = test_data[index].split(SPACE).collect();
            let depth = (depth_ln[0][1..]).parse::<u8>().unwrap_or(0) as i8;
            let expected_ln = depth_ln[1].parse::<u64>().unwrap_or(0);

            // Abort if depth parsing failed
            result = if depth == 0 {
                TestResult::ERR_DEPTH
            } else {
                result
            };

            // Abort if parsing expected number of leaf nodes failed
            result = if expected_ln == 0 {
                TestResult::ERR_EXPECT
            } else {
                result
            };

            if result == 0 {
                print!("Expect for depth {depth}: {expected_ln}");

                // This is the actual perft run for this test and depth.
                let now = Instant::now();
                let found_ln = perft::perft(
                    &mut board,
                    depth,
                    &move_generator,
                    &mut transposition,
                    tt_enabled,
                );
                let elapsed = now.elapsed().as_millis();
                let moves_per_second = ((found_ln * 1000) as f64 / elapsed as f64).floor();
                let is_ok = expected_ln == found_ln;

                // Print the results
                print!(" - Found: {found_ln}");
                print!(" - Result: {}", if is_ok { "OK" } else { "Fail" });
                println!(" ({elapsed} ms, {moves_per_second} leaves/sec)");

                result = if !is_ok { TestResult::ERR_FAIL } else { result };
            }

            index += 1;
        }

        println!("Test {}: {}\n", test_nr + 1, TEST_RESULTS[result]);
        test_nr += 1;
    }
}

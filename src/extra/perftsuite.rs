use super::large_epd::LARGE_PERFT_SUITE;
use crate::{
    board::{defs::ZobristRandoms, Board},
    extra::{perft, print},
    movegen::MoveGenerator,
};
use std::{sync::Arc, time::Instant};

const SEMI_COLON: char = ';';
const SPACE: char = ' ';

// This function runs the entire suite.
#[allow(dead_code)]
pub fn run_all_tests() {
    let entire_suite = &LARGE_PERFT_SUITE[..];
    run(entire_suite);
}

// This function runs a single test.
#[allow(dead_code)]
pub fn run_single_test(test: usize) {
    let length = LARGE_PERFT_SUITE.len();
    if (1..=length).contains(&test) {
        let single_test = &LARGE_PERFT_SUITE[(test - 1)..test];
        run(single_test);
    } else {
        println!("Perftsuite: Test does not exist. Range: 1 - {}", length);
    }
}

// This private function is the one actually running tests.
// This can be the entire suite, or a single test.
#[allow(dead_code)]
fn run(subset: &[&str]) {
    let number_of_tests = subset.len();
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();
    let mut board: Board = Board::new(Arc::new(zobrist_randoms), Arc::new(move_generator));
    let mut abort = false;

    // Run all the tests.
    for (test_nr, test) in subset.iter().enumerate() {
        // Split the test's data string into multiple parts.
        let data: Vec<String> = test
            .split(SEMI_COLON)
            .map(|s| s.trim().to_string())
            .collect();
        let fen = &data[0];

        let setup_result = board.fen_read(Some(fen));
        println!("Test {} from {}", test_nr + 1, number_of_tests);
        println!("FEN: {}", fen);

        // If setup ok, then print position. Else, print error and continue to the next test.
        match setup_result {
            Ok(()) => print::position(&board, None),
            Err(e) => {
                println!("Error in FEN-string part: {}", e);
                continue;
            }
        };

        // Run each test at the given depths.
        for (i, d) in data.iter().enumerate() {
            // Data index 0 contains the FEN-string, so skip this and
            // start at index 1 to find the expected leaf nodes per depth.
            if i > 0 {
                // Split "D1 20" into a vector containing "D1" (depth) and "20" (leaf nodes)
                let depth_ln: Vec<String> = d.split(SPACE).map(|s| s.to_string()).collect();

                // Parse the depth to an integer. Skip the first character "D".
                let depth = (depth_ln[0][1..]).parse::<u8>().unwrap();

                // Parse the number of leaf nodes to an integer.
                let expected_leaf_nodes = depth_ln[1].parse::<u64>().unwrap();

                print!("Expect for depth {}: {}", depth, expected_leaf_nodes);

                // This is the actual perft run for this test and depth.
                let now = Instant::now();
                let found_leaf_nodes = perft::perft(&mut board, depth);
                let elapsed = now.elapsed().as_millis();
                let moves_per_second = ((found_leaf_nodes * 1000) as f64 / elapsed as f64).floor();
                let is_ok = expected_leaf_nodes == found_leaf_nodes;
                let result = if is_ok { "OK" } else { "Error" };

                // Print the results
                print!(" - Found: {}", found_leaf_nodes);
                print!(" - Result: {}", result);
                println!(" ({} ms, {} moves/sec)", elapsed, moves_per_second);

                abort = expected_leaf_nodes != found_leaf_nodes;

                // Break if there are errors.
                if abort {
                    break;
                }
            }
        }

        if !abort {
            println!("Test {} finished without errors.", test_nr + 1);
            println!();
        } else {
            println!("Test {} stopped because of errors.", test_nr + 1);
            println!();
            break;
        }
    }

    if !abort {
        println!("Perft testing completed without errors.");
    } else {
        println!("Perft testing found errors.");
    }
}

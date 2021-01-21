/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use crate::{
    board::Board,
    extra::epds::LARGE_TEST_EPDS,
    misc::{perft, print},
    movegen::MoveGenerator,
};
use std::time::Instant;

const SEMI_COLON: char = ';';
const SPACE: char = ' ';

const ERR_NONE: usize = 0;
const ERR_FEN: usize = 1;
const ERR_DEPTH: usize = 2;
const ERR_EXPECT: usize = 3;
const ERR_FAIL: usize = 4;

const TEST_RESULTS: [&str; 5] = [
    "No errors. Test completed successfully.",
    "Errors in parsing the FEN-string.",
    "Errors parsing depth from test data.",
    "Errors parsing expected leaf nodes from test data.",
    "Failure: Found leaf nodes not equal to expected value.",
];

// This private function is the one actually running tests.
// This can be the entire suite, or a single test.
pub fn run() {
    let number_of_tests = LARGE_TEST_EPDS.len();
    let move_generator = MoveGenerator::new();
    let mut board: Board = Board::new();
    let mut result: usize = ERR_NONE;

    // Run all the tests.
    let mut test_nr = 0;
    while (test_nr < number_of_tests) && (result == 0) {
        // Split the test's data string into multiple parts.
        let test_data: Vec<String> = LARGE_TEST_EPDS[test_nr]
            .split(SEMI_COLON)
            .map(|s| s.trim().to_string())
            .collect();
        let fen = &test_data[0];

        // Set up the position according to the provided FEN-string.
        let setup_result = board.fen_read(Some(fen));
        println!("Test {} from {}", test_nr + 1, number_of_tests);
        println!("FEN: {}", fen);

        // If setup ok, then print position. Else, print error and continue to the next test.
        match setup_result {
            Ok(()) => print::position(&board, None),
            Err(_) => result = ERR_FEN,
        };

        // Run all the parts of a test.
        let mut index: usize = 1;
        while index < test_data.len() && (result == 0) {
            // Data index 0 contains the FEN-string, so skip this and
            // start at index 1 to find the expected leaf nodes per depth.

            // Split "D1 20" into a vector containing "D1" (depth) and "20" (leaf nodes)
            let depth_ln: Vec<String> = test_data[index]
                .split(SPACE)
                .map(|s| s.to_string())
                .collect();

            let depth = (depth_ln[0][1..]).parse::<u8>().unwrap_or(0);
            let expected_ln = depth_ln[1].parse::<u64>().unwrap_or(0);

            // Abort if depth or expected leaf node parsing fails.
            result = if depth == 0 { ERR_DEPTH } else { result };
            result = if expected_ln == 0 { ERR_EXPECT } else { result };

            if result == 0 {
                print!("Expect for depth {}: {}", depth, expected_ln);

                // This is the actual perft run for this test and depth.
                let now = Instant::now();
                let found_ln = perft::perft(&mut board, depth, &move_generator);
                let elapsed = now.elapsed().as_millis();
                let moves_per_second = ((found_ln * 1000) as f64 / elapsed as f64).floor();
                let is_ok = expected_ln == found_ln;

                // Print the results
                print!(" - Found: {}", found_ln);
                print!(" - Result: {}", if is_ok { "OK" } else { "Fail" });
                println!(" ({} ms, {} leaves/sec)", elapsed, moves_per_second);

                result = if !is_ok { ERR_FAIL } else { result };
            }

            index += 1;
        }

        println!("Test {}: {}\n", test_nr + 1, TEST_RESULTS[result]);
        test_nr += 1;
    }
}

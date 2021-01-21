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

mod board;
mod comm;
mod defs;
mod engine;
mod evaluation;
mod misc;
mod movegen;
mod search;

#[cfg(feature = "extra")]
mod extra;

// use interface::console;
use defs::ENGINE_RUN_ERRORS;
use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    let result = engine.run();

    match result {
        Ok(()) => (),
        Err(e) => println!("Error code {}: {}", e, ENGINE_RUN_ERRORS[e as usize]),
    };
}

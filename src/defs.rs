/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2020, Marcel Vanthoor

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

// defs.rs in the root holds basic definitions. Any definitions needed
// within specific modules, are defined in defs.rs in the directory for
// that module.

pub struct About;
impl About {
    pub const ENGINE: &'static str = "Rustic";
    pub const VERSION: &'static str = "Alpha 1";
    pub const AUTHOR: &'static str = "Marcel Vanthoor";
    pub const EMAIL: &'static str = "mail@marcelvanthoor.nl";
    pub const WEBSITE: &'static str = "https://rustic-chess.org/";
}

pub type Bitboard = u64;
pub type Piece = usize;
pub type Side = usize;
pub type Square = usize;

#[derive(Copy, Clone, PartialEq)]
pub struct Sides;
impl Sides {
    pub const WHITE: Side = 0;
    pub const BLACK: Side = 1;
    pub const BOTH: Side = 2;
}

pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const FEN_KIWIPETE_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

pub struct NrOf;
impl NrOf {
    pub const PIECE_TYPES: usize = 6;
    pub const CASTLING_PERMISSIONS: usize = 16; // 0-15
    pub const SQUARES: usize = 64;
    pub const FILES: usize = 8;
    pub const RANKS: usize = 8;
}

pub struct Castling;
impl Castling {
    pub const WK: u8 = 1;
    pub const WQ: u8 = 2;
    pub const BK: u8 = 4;
    pub const BQ: u8 = 8;
    pub const ALL: u8 = 15;
}

pub const EMPTY: u64 = 0;
pub const MAX_GAME_MOVES: usize = 2048;
pub const MAX_LEGAL_MOVES: u8 = 255;
pub const MAX_DEPTH: u8 = 254;
pub const MAX_MOVE_RULE: u8 = 100; // 50/75 move rule

// Define errors
pub type EngineRunResult = Result<(), u8>;
pub const ENGINE_RUN_ERRORS: [&str; 7] = [
    "FEN: Must have six parts",
    "FEN: Pieces and squares incorrect",
    "FEN: Color selection incorrect",
    "FEN: Castling permissions incorrect",
    "FEN: En-passant square incorrect",
    "FEN: Half-move clock incorrect",
    "FEN: Full-move number incorrect",
];

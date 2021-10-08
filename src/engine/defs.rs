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

pub use crate::engine::transposition::{HashFlag, IHashData, PerftData, SearchData, TT};
use crate::{comm::CommInput, search::defs::SearchReport};

#[derive(PartialEq, Copy, Clone)]
pub enum EngineStatus {
    Idle,
    Analyzing,
    Searching,
}

// This struct holds messages that are reported on fatal engine errors.
// These should never happen; if they do the engine is in an unknown state,
// and it will panic without trying any recovery whatsoever.
pub struct ErrFatal;
impl ErrFatal {
    pub const CREATE_COMM: &'static str = "Comm creation failed.";
    pub const NEW_GAME: &'static str = "Setting up new game failed.";
    pub const LOCK: &'static str = "Lock failed.";
    pub const READ_IO: &'static str = "Reading I/O failed.";
    pub const HANDLE: &'static str = "Broken handle.";
    pub const THREAD: &'static str = "Thread has failed.";
    pub const CHANNEL: &'static str = "Broken channel.";
    pub const NO_INFO_RX: &'static str = "No incoming Info channel.";
}

pub struct ErrNormal;
impl ErrNormal {
    pub const NOT_LEGAL: &'static str = "This is not a legal move in this position.";
    pub const NOT_INT: &'static str = "The value given was not an integer.";
    pub const FEN_FAILED: &'static str = "Setting up FEN failed. Board not changed.";
}

// This struct holds the engine's settings.
pub struct Settings {
    pub threads: usize,
    pub quiet: bool,
    pub tt_size: usize,
}

// This enum provides information to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq)]
pub enum Information {
    Comm(CommInput),
    Search(SearchReport),
}

pub enum UiElement {
    Spin,
    Button,
}

pub struct EngineOption {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl EngineOption {
    pub fn new(
        name: &'static str,
        ui_element: UiElement,
        default: Option<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self {
            name,
            ui_element,
            default,
            min,
            max,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum EngineSetOption {
    Hash(String),
    ClearHash,
    Nothing,
}
impl EngineSetOption {
    pub const HASH: &'static str = "Hash";
    pub const CLEAR_HASH: &'static str = "Clear Hash";
}

pub struct EngineOptionDefaults;
impl EngineOptionDefaults {
    pub const HASH_DEFAULT: &'static str = "32";
    pub const HASH_MIN: &'static str = "0";
    pub const HASH_MAX_64_BIT: &'static str = "65536";
    pub const HASH_MAX_32_BIT: &'static str = "2048";
}

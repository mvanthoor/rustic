/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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

// Move sorting routines.

use super::{
    defs::{SearchRefs, MAX_KILLER_MOVES},
    Search,
};
use crate::{board::defs::Pieces, defs::NrOf, movegen::defs::MoveList, movegen::defs::ShortMove};

const MVV_LVA_OFFSET: u32 = u32::MAX - 256;
const TTMOVE_SORT_VALUE: u32 = 60;
const KILLER_VALUE: u32 = 10;
const COUNTER_VALUE: u32 = 15;

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u16; NrOf::PIECE_TYPES + 1]; NrOf::PIECE_TYPES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim K, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

impl Search {
    pub fn score_moves(ml: &mut MoveList, tt_move: ShortMove, refs: &SearchRefs) {
        for i in 0..ml.len() {
            let m = ml.get_mut_move(i);
            let mut value: u32 = 0;

            // Sort order priority is: TT Move first, then captures, then
            // quiet moves that are in the list of killer moves.
            if m.get_move() == tt_move.get_move() {
                value = MVV_LVA_OFFSET + TTMOVE_SORT_VALUE;
            } else if m.captured() != Pieces::NONE {
                // Order captures higher than MVV_LVA_OFFSET
                value = MVV_LVA_OFFSET + MVV_LVA[m.captured()][m.piece()] as u32;
            } else {
                let ply = refs.search_info.ply as usize;
                let mut n = 0;
                while n < MAX_KILLER_MOVES && value == 0 {
                    let killer = refs.search_info.killer_moves[ply][n];
                    if m.get_move() == killer.get_move() {
                        // Order killers below MVV_LVA_OFFSET
                        value = MVV_LVA_OFFSET - ((i as u32 + 1) * KILLER_VALUE);
                    }
                    n += 1;
                }

                if value == 0 && refs.board.history.len() > 0 {
                    let prev = refs.board.history.get_ref(refs.board.history.len() - 1).next_move;
                    let cm = refs.search_info.counter_moves[refs.board.us()][prev.piece()][prev.to()];
                    if m.get_move() == cm.get_move() {
                        value = MVV_LVA_OFFSET - ((i as u32 + 1) * COUNTER_VALUE);
                    }
                }
            }

            
            // If still not sorted, try to sort by history heuristic.
            if value == 0 {
                let piece = m.piece();
                let to = m.to();
                value = refs.search_info.history_heuristic[refs.board.us()][piece][to];
            }
            

            m.set_sort_score(value);
        }
    }

    // This function puts the move with the highest sort score at the
    // "start_index" position, where alpha-beta will pick the next move.
    pub fn pick_move(ml: &mut MoveList, start_index: u8) {
        for i in (start_index + 1)..ml.len() {
            if ml.get_move(i).get_sort_score() > ml.get_move(start_index).get_sort_score() {
                ml.swap(start_index as usize, i as usize);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        engine::defs::{Information, SearchData, TT},
        movegen::{MoveGenerator, defs::{MoveList, MoveType}},
        search::defs::{SearchControl, SearchInfo, SearchParams, SearchRefs},
    };
    use crossbeam_channel::unbounded;
    use std::sync::{Arc, RwLock};

    #[test]
    fn history_heuristic_affects_scoring() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(0)));
        let (_ct, crx) = unbounded::<SearchControl>();
        let (rtx, _rrx) = unbounded::<Information>();
        let mut sp = SearchParams::new();
        let mut si = SearchInfo::new();

        board.fen_read(None).unwrap();
        let mut ml = MoveList::new();
        mg.generate_moves(&board, &mut ml, MoveType::All);

        assert!(ml.len() > 1);
        let mv0 = ml.get_move(0);
        let side = board.us();

        let refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: false,
            search_params: &mut sp,
            search_info: &mut si,
            control_rx: &crx,
            report_tx: &rtx,
        };

        refs.search_info.history_heuristic[side][mv0.piece()][mv0.to()] = 500;

        Search::score_moves(&mut ml, ShortMove::new(0), &refs);
        Search::pick_move(&mut ml, 0);

        assert_eq!(ml.get_move(0).get_move(), mv0.get_move());
    }
}
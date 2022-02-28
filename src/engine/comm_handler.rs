/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
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

mod uci;
mod xboard;

use crate::{
    board::defs::{Pieces, BB_SQUARES, PIECE_NAME},
    comm::defs::{CommIn, CommOut},
    defs::Sides,
    engine::{
        defs::{ErrFatal, ErrNormal, Messages},
        Engine,
    },
    evaluation::Evaluation,
    misc::parse,
};

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_handler(&mut self, input: &CommIn) {
        match input {
            CommIn::Uci(command) => self.uci_handler(command),
            CommIn::XBoard(command) => self.xboard_handler(command),

            CommIn::Quit => self.quit(),
            CommIn::Board => self.comm.send(CommOut::PrintBoard),
            CommIn::History => self.comm.send(CommOut::PrintHistory),

            CommIn::Eval => {
                let mtx_board = &self.board.lock().expect(ErrFatal::LOCK);
                let eval = Evaluation::evaluate_position(mtx_board);
                let phase = mtx_board.game_state.phase_value;
                self.comm.send(CommOut::PrintEval(eval, phase));
            }

            CommIn::State => self.comm.send(CommOut::PrintState(self.state)),

            CommIn::ClearTt => {
                self.tt_search.lock().expect(ErrFatal::LOCK).clear();
                self.comm
                    .send(CommOut::Message(Messages::CLEARED_TT.to_string()));
            }

            CommIn::Bitboards(algebraic_square) => {
                let square = parse::algebraic_square_to_number(algebraic_square);
                if let Some(square) = square {
                    let mtx_board = self.board.lock().expect(ErrFatal::LOCK);
                    let piece = mtx_board.piece_list[square];

                    if piece != Pieces::NONE {
                        let white = (mtx_board.bb_side[Sides::WHITE] & BB_SQUARES[square]) > 0;
                        let side = if white { Sides::WHITE } else { Sides::BLACK };
                        let color = if white { "White" } else { "Black" };
                        let own_pieces = if white {
                            mtx_board.bb_side[Sides::WHITE]
                        } else {
                            mtx_board.bb_side[Sides::BLACK]
                        };
                        let attacks = match piece {
                            Pieces::KING | Pieces::KNIGHT => {
                                self.mg.get_non_slider_attacks(piece, square)
                            }
                            Pieces::QUEEN | Pieces::ROOK | Pieces::BISHOP => self
                                .mg
                                .get_slider_attacks(piece, square, mtx_board.occupancy()),
                            Pieces::PAWN => self.mg.get_pawn_attacks(side, square),
                            _ => panic!("Not a piece."),
                        };
                        let bitboard = attacks & !own_pieces;

                        println!("Found: {color} {}", PIECE_NAME[piece]);
                        println!("Bitboard: {bitboard}");
                    } else {
                        println!("Square is empty.");
                    }
                } else {
                    self.comm.send(CommOut::Error(
                        ErrNormal::PARAMETER_INVALID,
                        algebraic_square.to_string(),
                    ));
                }
            }

            CommIn::Help => self.comm.send(CommOut::PrintHelp),

            CommIn::Ignore(cmd) => {
                self.comm.send(CommOut::Message(format!(
                    "{}: {}",
                    Messages::COMMAND_IGNORED,
                    cmd
                )));
            }

            CommIn::Unknown(cmd) => self
                .comm
                .send(CommOut::Error(ErrNormal::UNKNOWN_COMMAND, cmd.to_string())),
        }
    }
}

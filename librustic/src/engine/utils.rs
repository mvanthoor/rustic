use crate::{
    defs::FEN_KIWIPETE_POSITION,
    engine::{defs::ErrFatal, Engine},
    misc::parse::{self, ConvertedMove},
    movegen::defs::{Move, MoveList, MoveType},
};
use if_chain::if_chain;

impl Engine {
    pub fn determine_startup_position(&mut self) -> String {
        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let fen_string = self.cmdline.fen();
        let kiwi_pete = self.cmdline.has_kiwipete();
        let kiwi_string = FEN_KIWIPETE_POSITION.to_string();

        if kiwi_pete {
            return kiwi_string;
        }

        fen_string
    }

    // This function executes a move on the internal board, if it legal to
    // do so in the given position.
    pub fn execute_move(&mut self, m: &str) -> bool {
        // Prepare shorthand variables.
        let empty_move = (0usize, 0usize, 0usize);
        let converted_move = parse::algebraic_move_to_numbers(m).unwrap_or(empty_move);
        let pseudo_legal_move = self.is_pseudo_legal_move(converted_move);

        if let Some(m) = pseudo_legal_move {
            return self.board.lock().expect(ErrFatal::LOCK).make(m, &self.mg);
        } else {
            false
        }
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually in the list of pseudo-legal moves for this position.
    pub fn is_pseudo_legal_move(&self, converted_move: ConvertedMove) -> Option<Move> {
        // Get the pseudo-legal move list for this position.
        let mut move_list = MoveList::new();
        self.mg.generate_moves(
            &self.board.lock().expect(ErrFatal::LOCK),
            &mut move_list,
            MoveType::All,
        );

        // Determine if the converted move is a possible pseudo-legal move
        // in the current position. make() wil determine final legality
        // when executing the move.
        for i in 0..move_list.len() {
            let m = move_list.get_move(i);
            if_chain! {
                if converted_move.0 == m.from();
                if converted_move.1 == m.to();
                if converted_move.2 == m.promoted();
                then {
                    return Some(m);
                }
            }
        }

        None
    }
}

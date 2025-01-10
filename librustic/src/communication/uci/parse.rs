use crate::{communication::uci::cmd_in::UciIn, defs::FEN_START_POSITION};

pub fn position(cmd: &str) -> UciIn {
    enum Tokens {
        Nothing,
        Fen,
        Moves,
    }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut fen = String::from("");
    let mut moves: Vec<String> = Vec::new();
    let mut skip_fen = false;
    let mut token = Tokens::Nothing;

    for p in parts {
        match p {
            "position" => (),
            "startpos" => skip_fen = true,
            "moves" => token = Tokens::Moves,
            "fen" => {
                if !skip_fen {
                    token = Tokens::Fen;
                }
            }
            _ => match token {
                Tokens::Nothing => (),
                Tokens::Fen => {
                    fen.push_str(p);
                    fen.push(' ');
                }
                Tokens::Moves => moves.push(String::from(p)),
            },
        }
    }
    // No FEN part in the command. Use the start position.
    if fen.is_empty() {
        fen = String::from(FEN_START_POSITION)
    }

    UciIn::Position(fen.trim().to_string(), moves)
}

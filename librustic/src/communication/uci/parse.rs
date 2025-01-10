use crate::{communication::uci::cmd_in::UciIn, defs::FEN_START_POSITION, search::defs::GameTime};

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

pub fn go(cmd: &str) -> UciIn {
    enum Tokens {
        Nothing,
        Depth,
        Nodes,
        MoveTime,
        WTime,
        BTime,
        WInc,
        BInc,
        MovesToGo,
    }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let mut received = UciIn::Unknown(cmd.to_string());
    let mut token = Tokens::Nothing;
    let mut game_time = GameTime::new(0, 0, 0, 0, None);

    for p in parts {
        match p {
            "go" => received = UciIn::GoInfinite,
            "infinite" => break, // Already Infinite; nothing more to do.
            "depth" => token = Tokens::Depth,
            "movetime" => token = Tokens::MoveTime,
            "nodes" => token = Tokens::Nodes,
            "wtime" => token = Tokens::WTime,
            "btime" => token = Tokens::BTime,
            "winc" => token = Tokens::WInc,
            "binc" => token = Tokens::BInc,
            "movestogo" => token = Tokens::MovesToGo,
            _ => match token {
                Tokens::Nothing => (),
                Tokens::Depth => {
                    let depth = p.parse::<i8>().unwrap_or(1);
                    received = UciIn::GoDepth(depth);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::MoveTime => {
                    let milliseconds = p.parse::<u128>().unwrap_or(1000);
                    received = UciIn::GoMoveTime(milliseconds);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::Nodes => {
                    let nodes = p.parse::<usize>().unwrap_or(1);
                    received = UciIn::GoNodes(nodes);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::WTime => game_time.wtime = p.parse::<u128>().unwrap_or(0),
                Tokens::BTime => game_time.btime = p.parse::<u128>().unwrap_or(0),
                Tokens::WInc => game_time.winc = p.parse::<u128>().unwrap_or(0),
                Tokens::BInc => game_time.binc = p.parse::<u128>().unwrap_or(0),
                Tokens::MovesToGo => {
                    game_time.moves_to_go = if let Ok(x) = p.parse::<usize>() {
                        Some(x)
                    } else {
                        None
                    }
                }
            },
        }
    }

    // If we are still in the default "go infinite" mode, we must switch to
    // GameTime mode if at least one parameter of "go wtime btime winc
    // binc" was set to something else but 0.
    let is_default_mode = received == UciIn::GoInfinite;
    let has_time = game_time.wtime > 0 || game_time.btime > 0;
    let has_inc = game_time.winc > 0 || game_time.binc > 0;
    let is_game_time = has_time || has_inc;

    if is_default_mode && is_game_time {
        received = UciIn::GoGameTime(game_time);
    }

    received
}

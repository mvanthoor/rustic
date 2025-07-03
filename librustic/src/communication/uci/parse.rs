use crate::communication::uci::defs::{Name, Value};
use crate::{communication::uci::cmd_in::UciIn, defs::FEN_START_POSITION, search::defs::GameTime};

const SPACE: char = ' ';

pub fn position(cmd: &str) -> UciIn {
    enum Tokens {
        Empty,
        Fen,
        Moves,
    }

    let stream: Vec<&str> = cmd.split_whitespace().collect();
    let mut fen = String::from("");
    let mut moves: Vec<String> = Vec::new();
    let mut skip_fen = false;
    let mut token = Tokens::Empty;

    for s in stream {
        match s {
            "position" => (),
            "startpos" => skip_fen = true,
            "moves" => token = Tokens::Moves,
            "fen" => {
                if !skip_fen {
                    token = Tokens::Fen;
                }
            }
            _ => match token {
                Tokens::Empty => (),
                Tokens::Fen => {
                    fen.push_str(s);
                    fen.push(SPACE);
                }
                Tokens::Moves => moves.push(String::from(s)),
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
        Empty,
        Depth,
        Nodes,
        MoveTime,
        WTime,
        BTime,
        WInc,
        BInc,
        MovesToGo,
    }

    let stream: Vec<&str> = cmd.split_whitespace().collect();
    let mut received = UciIn::Unknown(cmd.to_string());
    let mut token = Tokens::Empty;
    let mut game_time = GameTime::new(0, 0, 0, 0, None);

    for s in stream {
        match s {
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
                Tokens::Empty => (),
                Tokens::Depth => {
                    let depth = s.parse::<i8>().unwrap_or(1);
                    received = UciIn::GoDepth(depth);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::MoveTime => {
                    let milliseconds = s.parse::<u128>().unwrap_or(1000);
                    received = UciIn::GoMoveTime(milliseconds);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::Nodes => {
                    let nodes = s.parse::<usize>().unwrap_or(1);
                    received = UciIn::GoNodes(nodes);
                    break; // break for-loop: nothing more to do.
                }
                Tokens::WTime => game_time.wtime = s.parse::<u128>().unwrap_or(0),
                Tokens::BTime => game_time.btime = s.parse::<u128>().unwrap_or(0),
                Tokens::WInc => game_time.winc = s.parse::<u128>().unwrap_or(0),
                Tokens::BInc => game_time.binc = s.parse::<u128>().unwrap_or(0),
                Tokens::MovesToGo => game_time.moves_to_go = s.parse::<usize>().ok(),
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

pub fn setoption(cmd: &str) -> UciIn {
    enum Tokens {
        Empty,
        Name,
        Value,
    }

    let stream: Vec<&str> = cmd.split_whitespace().collect();
    let mut token = Tokens::Empty;
    let mut name: Name = String::from("");
    let mut value: Value = None;

    for s in stream {
        match s {
            "setoption" => (),
            "name" => token = Tokens::Name,
            "value" => token = Tokens::Value,
            _ => match token {
                Tokens::Name => {
                    name.push_str(s);
                    name.push(SPACE);
                }
                Tokens::Value => value = Some(s.to_lowercase()),
                Tokens::Empty => (),
            },
        }
    }

    UciIn::SetOption(name.trim().to_lowercase(), value)
}

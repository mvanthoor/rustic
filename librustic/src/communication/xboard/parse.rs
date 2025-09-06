use super::cmd_in::XBoardIn;

pub fn key_value_pair(cmd: &str) -> XBoardIn {
    const KEY: usize = 0;
    const VALUE: usize = 1;
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    if parts.len() != 2 {
        return XBoardIn::Unknown(String::from(cmd));
    }

    match parts[KEY] {
        "protover" => XBoardIn::Protover(parts[VALUE].parse::<u8>().unwrap_or(0)),
        "ping" => XBoardIn::Ping(parts[VALUE].parse::<isize>().unwrap_or(0)),
        _ => XBoardIn::Unknown(String::from(cmd)),
    }
}

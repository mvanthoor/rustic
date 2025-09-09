use crate::communication::xboard::cmd_in::XBoardIn;

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
        "usermove" => {
            XBoardIn::Usermove(parts[VALUE].parse::<String>().unwrap_or(String::from("")))
        }
        _ => XBoardIn::Unknown(String::from(cmd)),
    }
}

pub fn setboard(cmd: &str) -> XBoardIn {
    let value = cmd.replace("setboard", "").trim().to_string();
    XBoardIn::SetBoard(value)
}

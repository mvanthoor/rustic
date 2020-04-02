pub fn strip_newline(input: &mut String) {
    for _ in 0..2 {
        let c = input.chars().next_back();
        let cr = if let Some('\r') = c { true } else { false };
        let lf = if let Some('\n') = c { true } else { false };

        if cr || lf {
            input.pop();
        }
    }
}

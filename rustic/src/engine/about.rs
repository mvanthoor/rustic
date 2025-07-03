use crate::engine::{defs::Settings, Engine};

pub const ENGINE: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "Marcel Vanthoor";
pub const EMAIL: &str = "mail@marcelvanthoor.nl";
pub const WEBSITE: &str = "https://rustic-chess.org/";

// This notice is displayed if the engine is a debug binary. (Debug
// binaries are unoptimized and slower than release binaries.)
#[cfg(debug_assertions)]
const NOTICE_DEBUG_MODE: &str = "Notice: Running in debug mode";

impl Engine {
    // Print information about the engine.
    pub fn print_fancy_about(&self, s: &Settings, protocol: &str) {
        let bits = std::mem::size_of::<usize>() * 8;

        Engine::print_ascii_logo();

        println!("{:<10} {} {}", "Engine:", ENGINE, VERSION);
        println!("{:<10} {}", "Author:", AUTHOR);
        println!("{:<10} {}", "EMail:", EMAIL);
        println!("{:<10} {}", "Website:", WEBSITE);
        println!("{:<10} {}-bit", "Type:", bits);
        println!("{:<10} {} MB", "TT size:", s.tt_size);
        println!("{:<10} {}", "Protocol:", protocol);

        if s.threads == 1 {
            println!("{:<10} {}", "Threads:", s.threads)
        } else {
            println!("{:<10} {} (unused, always 1)", "Threads:", s.threads)
        };

        #[cfg(debug_assertions)]
        println!("{NOTICE_DEBUG_MODE}");
        println!();
    }

    pub fn print_simple_about(&self, s: &Settings, protocol: &str) {
        println!(
            "{} {} | {} <{}> | TT: {} MB | {}",
            ENGINE, VERSION, AUTHOR, EMAIL, s.tt_size, protocol
        );

        #[cfg(debug_assertions)]
        println!("{NOTICE_DEBUG_MODE}");
    }

    fn print_ascii_logo() {
        println!();
        println!("d888888b                      dP   oo        ");
        println!("88     88                     88             ");
        println!("88oooo88  88    88  d8888b  d8888P dP d88888b");
        println!("88    88  88    88  8ooooo    88   88 88     ");
        println!("88     88 88    88       88   88   88 88     ");
        println!("88     88  88888P  888888P    dP   dP 888888P");
        println!("ooooooooooooooooooooooooooooooooooooooooooooo");
        println!();
    }
}

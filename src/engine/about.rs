use super::Engine;
use crate::defs::About;

// This notice is displayed if the engine is a debug binary. (Debug
// binaries are unoptimized and slower than release binaries.)
#[cfg(debug_assertions)]
const NOTICE_DEBUG_MODE: &str = "Notice: Running in debug mode";

impl Engine {
    pub fn print_ascii_logo(&self) {
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

    // Print information about the engine.
    pub fn print_about(&self) {
        println!("Engine: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {}", About::AUTHOR);
        println!("EMail: {}", About::EMAIL);
        println!("Website: {}", About::WEBSITE);
    }

    pub fn print_settings(&self, threads: usize, protocol: &str) {
        println!("Protocol: {}", protocol);
        println!("Threads: {}", threads);
        #[cfg(debug_assertions)]
        println!("{}", NOTICE_DEBUG_MODE);
    }
}

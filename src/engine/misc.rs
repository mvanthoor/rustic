use super::Engine;
use crate::defs::About;

// This notice is displayed if the engine is a debug binary. (Debug
// binaries are unoptimized and slower than release binaries.)
#[cfg(debug_assertions)]
const NOTICE_DEBUG_MODE: &'static str = "Notice: Running in debug mode";

impl Engine {
    // Print information about the engine.
    pub fn about(&self) {
        println!("Program: {} {}", About::ENGINE, About::VERSION);
        println!("Author: {} <{}>", About::AUTHOR, About::EMAIL);
        println!("Description: {}", About::DESCRIPTION);
        println!(
            "Threads: {} (not used yet, always 1)",
            self.settings.threads
        );
        println!("Protocol: {}", self.comm.get_protocol_name());

        #[cfg(debug_assertions)]
        println!("{}", NOTICE_DEBUG_MODE);
    }
}

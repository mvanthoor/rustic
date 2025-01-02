use crate::{
    board::Board,
    engine::defs::{EngineState, GameResult, Information},
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};
use std::sync::{mpsc::Sender, Arc, Mutex};

pub use crate::comm::protocols::{
    uci::{Uci, UciIn, UciOut},
    xboard::{TimeControl, XBoard, XBoardIn, XBoardOut},
};

pub enum UiElement {
    Spin,
    Button,
}

pub struct CommOption {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl CommOption {
    pub fn new(
        name: &'static str,
        ui_element: UiElement,
        default: Option<String>,
        min: Option<String>,
        max: Option<String>,
    ) -> Self {
        Self {
            name,
            ui_element,
            default,
            min,
            max,
        }
    }
}

// These are the types of communication the engine is capable of.
pub struct CommType;

impl CommType {
    pub const XBOARD: &'static str = "xboard";
    pub const UCI: &'static str = "uci";
}

pub struct CommInfo {
    protocol_name: &'static str,
    supports_fancy_about: bool,
    requires_stateful_mode: bool,
    requires_game_result: bool,
    startup_state: EngineState,
}

impl CommInfo {
    pub fn new(
        protocol_name: &'static str,
        supports_fancy_about: bool,
        requires_stateful_mode: bool,
        requires_game_result: bool,
        startup_state: EngineState,
    ) -> Self {
        Self {
            protocol_name,
            supports_fancy_about,
            requires_stateful_mode,
            requires_game_result,
            startup_state,
        }
    }

    pub fn protocol_name(&self) -> &str {
        self.protocol_name
    }

    pub fn supports_fancy_about(&self) -> bool {
        self.supports_fancy_about
    }

    pub fn requires_stateful_mode(&self) -> bool {
        self.requires_stateful_mode
    }

    pub fn requires_game_result(&self) -> bool {
        self.requires_game_result
    }

    pub fn startup_state(&self) -> EngineState {
        self.startup_state
    }
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<CommOption>>,
    );
    fn send(&self, msg: CommOut);
    fn shutdown(&mut self);
    fn info(&self) -> &CommInfo;
}

#[derive(PartialEq, Eq, Clone)]
pub enum CommIn {
    Uci(UciIn),
    XBoard(XBoardIn),

    // Common incoming commands
    Quit,
    Unknown(String),

    // Custom
    Board,
    History,
    Eval,
    State,
    Help,
    ClearTt,

    // Ignore an incoming command on purpose
    Ignore(String),
}

pub enum CommOut {
    Uci(UciOut),
    XBoard(XBoardOut),

    // Common output for all protocols
    BestMove(Move, Option<GameResult>), // Transmit the engine's best move and result.
    SearchCurrMove(SearchCurrentMove),  // Transmit currently considered move.
    SearchSummary(SearchSummary),       // Transmit search information.
    SearchStats(SearchStats),           // Transmit search Statistics.
    Message(String),                    // Transmits a message to the GUI.
    Error(&'static str, String),        // Transmits an error message.
    Quit,                               // Terminates the output thread.

    // Output to screen when running in a terminal window.
    PrintBoard,
    PrintHistory,
    PrintEval(i16, i16),
    PrintState(EngineState),
    PrintHelp,
}

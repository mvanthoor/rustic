use crate::{
    communication::uci::{cmd_in::UciIn, cmd_out::UciOut},
    {board::Board, search::defs::SearchReport},
};
use std::sync::{mpsc::Sender, Arc, Mutex};

pub trait IComm {
    fn init(
        &mut self,
        cmd_in_tx: Sender<UciIn>,
        search_tx: Sender<SearchReport>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<Features>>,
    );
    fn send(&self, msg: UciOut);
    fn shutdown(&mut self);
}

pub struct Features {
    pub name: &'static str,
    pub ui_element: UiElement,
    pub default: Option<String>,
    pub min: Option<String>,
    pub max: Option<String>,
}

impl Features {
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

pub enum UiElement {
    Spin,
    Button,
}

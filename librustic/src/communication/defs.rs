use super::uci::cmd_in::UciIn;
use crate::communication::uci::cmd_out::UciOut;
use crate::{board::Board, search::defs::SearchReport};
use std::sync::{mpsc::Sender, Arc, Mutex};

pub trait IComm {
    fn init(
        &mut self,
        cmd_in_tx: Sender<UciIn>,
        search_tx: Sender<SearchReport>,
        board: Arc<Mutex<Board>>,
        options: Arc<Vec<CommOption>>,
    );
    fn send(&self, msg: UciOut);
    fn shutdown(&mut self);
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

pub enum UiElement {
    Spin,
    Button,
}

use crate::communication::feature::Feature;
use crate::communication::xboard::XBoard;
use std::sync::Arc;

impl XBoard {
    pub fn output_thread(&mut self, features: Arc<Vec<Feature>>) {}
}

use crate::{
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};

#[derive(PartialEq, Eq)]
pub enum UciOut {
    // UCI specification
    Id,
    ReadyOk,
    SearchCurrMove(SearchCurrentMove),
    SearchSummary(SearchSummary),
    SearchStats(SearchStats),
    BestMove(Move),
    InfoString(String),
    Quit,

    // Custom output
    Custom(String),
}

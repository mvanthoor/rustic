use crate::{
    movegen::defs::Move,
    search::defs::{SearchCurrentMove, SearchStats, SearchSummary},
};

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

    // Custom
    PrintBoard,
}

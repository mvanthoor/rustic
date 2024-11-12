use std::sync::{Arc, Mutex};

use rustic::{
    board::Board,
    engine::defs::{Information, SearchData, Verbosity, TT},
    movegen::MoveGenerator,
    search::{
        defs::{GameTime, SearchControl, SearchMode, SearchParams, SearchReport},
        Search,
    },
};

fn main() {
    // setup board
    let fen = "rnbqkbnr/pp1pppp1/8/2p4p/4P3/2P5/PP1P1PPP/RNBQKBNR w KQkq h6 0 3";
    let mut board = Board::new();
    board
        .fen_setup(Some(fen))
        .expect("failed to setup board from fen");
    let board = Arc::new(Mutex::new(board));

    // setup move generator
    let move_generator = Arc::new(MoveGenerator::new());

    // setup transposition table
    let tt_size = 32; // TODO: not sure
    let transposition_table = Arc::new(Mutex::new(TT::<SearchData>::new(tt_size)));

    // setup search
    let mut search = Search::new();
    let (info_tx, info_rx) = crossbeam_channel::unbounded::<Information>();
    search.init(info_tx, board, move_generator, transposition_table);

    // start search
    let depth = 8;
    search.send(SearchControl::Start(SearchParams {
        depth,
        game_time: GameTime::new(0, 0, 0, 0, None),
        move_time: 0,
        nodes: 0,
        search_mode: SearchMode::Depth,
        verbosity: Verbosity::Full,
    }));
    // wait for best move
    let best_move = loop {
        let info = info_rx.recv().expect("failed to receive info");
        let search_report = match info {
            Information::Search(search_report) => search_report,
            _ => panic!("expected search report"),
        };
        match search_report {
            SearchReport::Finished(best_move) => {
                println!("search finished");
                break best_move;
            }
            SearchReport::SearchSummary(search_summary) => {
                println!("search summary: {search_summary:?}");
            }
            SearchReport::SearchCurrentMove(search_current_move) => {
                println!("search current move: {search_current_move:?}");
            }
            SearchReport::SearchStats(search_stats) => {
                println!("search stats: {search_stats:?}");
            }
            SearchReport::Ready => {
                println!("search ready");
            }
        }
    };

    // print best move
    println!("best move: {best_move}");

    // cleanup
    search.send(SearchControl::Quit);
    search.shutdown();
}

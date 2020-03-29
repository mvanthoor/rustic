use crate::board::make_move::make_move;
use crate::board::representation::Board;
use crate::board::unmake_move::unmake_move;
use crate::board::zobrist::{ZobristKey, ZobristRandoms};
use crate::defs::SQUARE_NAME;
use crate::movegen::movedefs::MoveListPool;
use crate::movegen::MoveGenerator;
use crate::print;
use std::mem;
use std::time::Instant;

#[derive(Clone)]
pub struct PerftHashEntry {
    depth: u8,
    zobrist_key: ZobristKey,
    leaf_nodes: u64,
}

impl PerftHashEntry {
    pub fn new(d: u8, zk: ZobristKey, ln: u64) -> PerftHashEntry {
        PerftHashEntry {
            depth: d,
            zobrist_key: zk,
            leaf_nodes: ln,
        }
    }
}

pub struct PerftHashTable {
    hash_table: Vec<PerftHashEntry>,
    max_entries: u64,
    count: u64,
}

impl PerftHashTable {
    pub fn new(megabytes: u64) -> PerftHashTable {
        const ENTRY_SIZE: u64 = mem::size_of::<PerftHashEntry>() as u64;
        const ENTRIES_PER_MEGABYTE: u64 = (1024 * 1024 / ENTRY_SIZE);
        let entries = ENTRIES_PER_MEGABYTE * megabytes;
        PerftHashTable {
            hash_table: vec![PerftHashEntry::new(0, 0, 0); entries as usize],
            max_entries: entries,
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.hash_table = vec![PerftHashEntry::new(0, 0, 0); self.max_entries as usize];
        self.count = 0;
    }

    pub fn push(&mut self, entry: PerftHashEntry) {
        let index = (entry.zobrist_key % self.max_entries) as usize;
        // let ht_key = self.hash_table[index].zobrist_key;
        // let ht_depth = self.hash_table[index].depth;
        // if (ht_key == 0) || ((ht_key == entry.zobrist_key) && (ht_depth < entry.depth)) {
        //     self.hash_table[index] = entry;
        // }
        self.hash_table[index] = entry;
    }

    pub fn leaf_nodes(&self, depth: u8, zk: ZobristKey) -> Option<u64> {
        let index = (zk % self.max_entries) as usize;
        let correct_key = self.hash_table[index].zobrist_key == zk;
        let correct_depth = self.hash_table[index].depth == depth;
        if correct_key && correct_depth {
            Some(self.hash_table[index].leaf_nodes)
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub fn bench(depth: u8) {
    let mut total_time: u128 = 0;
    let mut total_nodes: u64 = 0;
    let mut hash_table: PerftHashTable = PerftHashTable::new(4096);
    let move_generator = MoveGenerator::new();
    let zobrist_randoms = ZobristRandoms::new();

    println!("Benchmarking perft 1-{} from starting position...", depth);

    for d in 1..=depth {
        let mut move_list_pool = MoveListPool::new();
        let mut perft_board: Board = Board::new(&zobrist_randoms, &move_generator, None);
        let now = Instant::now();
        let leaf_nodes = perft(&mut perft_board, d, &mut move_list_pool, &mut hash_table);
        let elapsed = now.elapsed().as_millis();
        let leaves_per_second = ((leaf_nodes * 1000) as f64 / elapsed as f64).floor();
        total_time += elapsed;
        total_nodes += leaf_nodes;
        println!(
            "Perft {}: {} ({} ms, {} leaves/sec)",
            d, leaf_nodes, elapsed, leaves_per_second
        );
    }

    let final_lnps = ((total_nodes * 1000) as f64 / total_time as f64).floor();
    println!("Total time spent: {} ms", total_time);
    println!("Execution speed: {} leaves/second", final_lnps);
}

#[allow(dead_code)]
pub fn perft(
    board: &mut Board,
    depth: u8,
    mlp: &mut MoveListPool,
    hash: &mut PerftHashTable,
) -> u64 {
    let mut leaf_nodes: u64 = 0;
    let index = depth as usize;

    if depth == 0 {
        return 1;
    }

    if let Some(ln) = hash.leaf_nodes(depth, board.zobrist_key) {
        ln
    } else {
        board.gen_all_moves(mlp.get_list_mut(index));
        for i in 0..mlp.get_list(index).len() {
            if !make_move(board, mlp.get_list(index).get_move(i)) {
                continue;
            };
            leaf_nodes += perft(board, depth - 1, mlp, hash);
            unmake_move(board);
        }
        hash.push(PerftHashEntry::new(depth, board.zobrist_key, leaf_nodes));

        leaf_nodes
    }
}

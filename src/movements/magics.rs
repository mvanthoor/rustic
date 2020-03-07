/**
 * This module provicdes "Magics", for working with sliding pieces in the move generator.
*/
extern crate rand; // Random number generator for creating magics.
use super::blockatt::{
    create_bishop_attack_boards, create_blocker_boards, create_rook_attack_boards,
};
use super::masks::{create_bishop_mask, create_rook_mask};
use super::{BISHOP_TABLE_SIZE, EMPTY, ROOK_TABLE_SIZE};
use crate::defines::{Bitboard, Piece, ALL_SQUARES, BISHOP, NR_OF_SQUARES, ROOK};
use crate::print;
use crate::print::PIECE_NAME;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/** Rook magic numbers. Don't touch them. Changing these numbers breaks the program. */
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
pub const ROOK_MAGICS: [u64; NR_OF_SQUARES as usize] = [
    324259448050975248u64, 162139001189302336u64, 4647750006529359880u64, 144121785691422736u64,
    16176938657641660544u64, 9367489423970945072u64, 36051338366288384u64, 36029147746665088u64,
    3518447965192208u64, 4614078830617822340u64, 9241949523864129664u64, 11540615780106252u64,
    730287067600519297u64, 144819425575437312u64, 1225261127674627584u64, 40814017656160512u64,
    594475700577118276u64, 283675082228259u64, 148058037853261952u64, 14411662294658320384u64,
    2394186703782912u64, 1157847866488718336u64, 2306407062973841412u64, 4576167411597460u64,
    2323857959626489888u64, 18860477004136448u64, 621497027752297522u64, 3027553647748714496u64,
    9241953785514295424u64, 1970363492082688u64, 1729664285938024960u64, 4836870457972064321u64,
    141012374650913u64, 4652253601601699840u64, 58687601506263040u64, 281543780081672u64,
    1157433900411130112u64, 81628378934806544u64, 2310366730829959192u64, 2900476768907429780u64,
    36558770110480u64, 9042384969023488u64, 180425597514743824u64, 5487636764434923528u64,
    5766860422494879764u64, 9224498487624761348u64, 41702298761822218u64, 45599234000551940u64,
    70370891935872u64, 19210671497487104u64, 387030266675328u64, 289215847808893056u64,
    576469550545240192u64, 1153216449143113729u64, 9350715278336u64, 288521763922764288u64,
    282782794268833u64, 595672521157161122u64, 436884352794689609u64, 9241667927690743809u64,
    5188428314494240769u64, 1157988067282792450u64, 1152939243166828548u64, 4611967569673330817u64,
];

/** Bishop magic numbers. Don't touch them. Changing these numbers breaks the program. */
#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
pub const BISHOP_MAGICS: [u64; NR_OF_SQUARES as usize] = [
    2310454429704290569u64, 37163502750244928u64, 145330200115150856u64, 573953659699200u64,
    9845999220824211456u64, 574016004032512u64, 10093699283674480640u64, 2306407060834902016u64,
    2883575003184432136u64, 1747410678824308864u64, 9259405249167245312u64, 936784527773139074u64,
    4629702641998381057u64, 201028145628315697u64, 4899992295377881088u64, 4630405483133404688u64,
    153474299838154784u64, 2286992943744036u64, 434597432802681416u64, 865817269052115456u64,
    9156750026475656u64, 599823317909770240u64, 4578375142474880u64, 2308525819264500224u64,
    18596057879421451u64, 18331093560345096u64, 2305880392877736000u64, 56602859688444160u64,
    5382084129205534724u64, 5767422822691897608u64, 283691220206592u64, 144398865845093376u64,
    1163523824685120u64, 20267333288223264u64, 325489801822240u64, 4755836425302245636u64,
    594475563668865152u64, 1162496335329427604u64, 9244765235704371236u64, 576667461564269056u64,
    146371454722771202u64, 426679365288452u64, 13724105480340736u64, 1152922330050364928u64,
    4620737202526097424u64, 1316476062695166464u64, 13981996823661781640u64, 12430506881068303489u64,
    5193780677221351424u64, 426612797737280u64, 37445932288049152u64, 1171147012042137601u64,
    504403227018657856u64, 4629845569785954560u64, 4686013077882208273u64, 1154056209263894528u64,
    613054853085794304u64, 9025075185721408u64, 9571249324951568u64, 10999715432448u64,
    290408795603472u64, 10664524198170591488u64, 5924513492108288u64, 90511840181764112u64,
];

/**
 * Magics contain the following data:
 * mask: A Rook or Bishop mask for the square the magic belongs to.
 * shift: This number is needed to create the magic index. It's "64 - (nr. of bits set 1 in mask)"
 * offset: contains the offset where the indexing of the square's attack boards begin.
 * magic: the magic number itself, used to create the magic index into the attack table.
*/
#[derive(Copy, Clone)]
pub struct Magics {
    pub mask: Bitboard,
    pub shift: u8,
    pub offset: u64,
    pub magic: u64,
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            mask: 0,
            shift: 0,
            offset: 0,
            magic: 0,
        }
    }
}

/**
 * get_index() is the actual function that gets the magic index into the attack table.
 * The attack table is a perfect hash. This means the following.
 * - A rook on A1 has 7 squares vertical and 7 squares horizontal movement.
 * - This is a total of 14 bits. However, if there are no pieces on A2-A6, or B1-G1, the rook
 *      can always see A8 and H1. This means that if there are no blockers on the file or rank,
 *      the rook can 'see' the square at the edge of the board. Therefore, the bits marking the
 *      edge of a ray are not counted. Thus, the rook on A1 has actually 12 bits set.
 * - These bits along the rank and file denote the possible position of blocking pieces.
 * - For 12 bits, there are 4096 possible configuration of blockers (2 to the power of 12).
 * - Thus, square A1 has 4096 blocker boards.
 * - The get_index() function receives a board occupancy when called.
 * - occupancy * self.mask (the mask for the piece on the square the magic belongs to) yields
 *      a blocker board.
 * - Each blocker board (configuration of blockers) goes with one attack board (the squares the)
 *      piece can actually attack). This attack board is in the attack table.
 * - The formula calculates WHERE in the attack table the blocker board is:
 *      (blockerboard * magic number) >> (64 - bits in mask) + offset
 * - For the rook on A1 the outcome will be an index of 0 - 4095:
 *      0 - 4095 because of 4096 possible blocker (and thus, attack board) permutations
 *      0 for offset, because A1 is the first square.
 * - So the index for a rook on B1 will start at 4096, and so on. (So B1's offset is 4096.)
 * - The "magic number" is called magic, because it generates a UNIQUE index for each attack
 *      board in the attack table, without any collisions; so the entire table is exactly
 *      filled. This is called a perfect hash.
 * - Finding the magics is a process of just trying random numbers, with the formula below, over
 * and over again until a number is found that generates unique indexes for all of the permutations
 * of attacks of the piece on a particular square. See the explanation for find_magics().
 */
impl Magics {
    pub fn get_index(&self, occupancy: Bitboard) -> usize {
        let blockerboard = occupancy & self.mask;
        ((blockerboard.wrapping_mul(self.magic) >> self.shift) + self.offset) as usize
    }
}

/*
    TODO: even more comments here.
    Finding magic numbers step by step.
    fail_low and fail_high: check if the generated index is within the expected offset.
    If this is not the case, there's a bug somewhere in this code, or in get_index().
*/
#[allow(dead_code)]
pub fn find_magics(piece: Piece) {
    assert!(piece == ROOK || piece == BISHOP, "Illegal piece: {}", 0);
    let is_rook = piece == ROOK;
    let mut rook_table: Vec<Bitboard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<Bitboard> = vec![EMPTY; BISHOP_TABLE_SIZE];
    let mut random = SmallRng::from_entropy();
    let mut offset = 0;
    let mut total_permutations = 0;

    println!("Finding magics for: {}", PIECE_NAME[piece]);
    for sq in ALL_SQUARES {
        let mask = if is_rook {
            create_rook_mask(sq)
        } else {
            create_bishop_mask(sq)
        };
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let end = offset + permutations - 1; // End point in the attack table.
        let blocker_boards = create_blocker_boards(mask);
        let attack_boards = if is_rook {
            create_rook_attack_boards(sq, &blocker_boards)
        } else {
            create_bishop_attack_boards(sq, &blocker_boards)
        };
        let mut try_this: Magics = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
        let mut attempts = 0; // Track needed attempts to find the magic.

        // Set up the new magic.
        try_this.mask = mask;
        try_this.shift = (64 - bits) as u8;
        try_this.offset = offset;
        total_permutations += permutations;
        while !found {
            attempts += 1; // Next attempt to find magic.
            found = true; // Assume this new magic will work.
            try_this.magic = random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>();
            for i in 0..permutations {
                let next = i as usize;
                let index = try_this.get_index(blocker_boards[next]);
                let table: &mut [Bitboard] = if is_rook {
                    &mut rook_table[..]
                } else {
                    &mut bishop_table[..]
                };
                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error.");
                    table[index] = attack_boards[next];
                } else {
                    for i in offset..=end {
                        table[i as usize] = EMPTY;
                    }
                    found = false;
                    break;
                }
            }
        }
        print::found_magic(sq, try_this, offset, end, attempts);
        offset += permutations; // Set offset for next square.
    }
    assert!(
        (total_permutations as usize)
            == if is_rook {
                ROOK_TABLE_SIZE
            } else {
                BISHOP_TABLE_SIZE
            },
        "Creating magics failed. Permuations were skiped."
    );
}

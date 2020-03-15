use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

pub fn initialize() {
    let mut random = SmallRng::from_seed([192; 16]);

    for i in 0..10 {
        let key = random.gen::<u64>();
        println!("Key {}: {}", i, key);
    }
}

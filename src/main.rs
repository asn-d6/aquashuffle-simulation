use std::collections::HashSet;
use std::cmp;
use rand::{seq::IteratorRandom, thread_rng};
use rand::rngs::ThreadRng;
use hashbrown::HashMap;

const VECTOR_LENGTH: usize = 2_usize.pow(14);

// Size of one local shuffle
const SHUFFLE_SIZE: usize = 128;

// Upper bound on number of shuffles in protocol execution
const MAX_SHUFFLES: usize = 4000;

// Number of repetitions over which the average benchmark outcomes are computed
const NUMBER_OF_REPETITIONS: usize = 3;

// Fraction of corrupted cards
const FRACTION_CORRUPTED_COMMITMENTS: f64 = 1.0/4.0;
const CORRUPTED_COMMITMENTS: usize = ((VECTOR_LENGTH as f64) * FRACTION_CORRUPTED_COMMITMENTS) as usize;

// Target Water level
const TARGET_EPS: f64 = 4.0 / (VECTOR_LENGTH as f64 * (1.0 - FRACTION_CORRUPTED_COMMITMENTS));

/// Distribute water in uncorrupted cups of a given batch
fn distribute_water(cups: &mut HashMap<usize, f64>, corrupted: &HashSet<usize>, rng: &mut ThreadRng) {
    let shuffled_batch: HashSet<usize> = HashSet::from_iter((0..VECTOR_LENGTH).choose_multiple(rng, SHUFFLE_SIZE));

    // Get set of honest indices that will get shuffled
    let honest_set = &shuffled_batch - corrupted;
    let num_honest = honest_set.len();
    if num_honest == 0 {
        println!("no honest commitment selected!");
        return;
    }

    // Find out how much total water we are distributing
    let mut total_water: f64 = 0.0;
    for v in honest_set.iter() {
        total_water += cups.get(v).unwrap_or(&0.0);
    }
    let avg_water = total_water / (num_honest as f64);

    // Pour water to all the cups
    for index in honest_set {
        cups.insert(index, avg_water);
    }
}

fn main() {
    let mut rng = thread_rng();

    println!("Parameters!");
    println!("--------------------------------------------------------------------------------");
    println!("Vector length: {}", VECTOR_LENGTH);
    println!("Local shuffle size: {}", SHUFFLE_SIZE);
    println!("Fraction of commitments corrupt: {} ({})", FRACTION_CORRUPTED_COMMITMENTS, CORRUPTED_COMMITMENTS);
    println!("Target eps error: {}", TARGET_EPS);
    println!("--------------------------------------------------------------------------------");

    // Object for computing averages later on
    let mut sum_succ_in_round: HashMap<usize, f64> = HashMap::new();
    for t in 0..MAX_SHUFFLES { // Initialize hashmap with a default value of zero
        sum_succ_in_round.insert(t, 0.0);
    }

    println!("\tRepetitions");
    println!("\t------------");

    for repetition in 0..NUMBER_OF_REPETITIONS {
        println!("\t{}/{}", repetition+1, NUMBER_OF_REPETITIONS);

        // Flag to be set, when sufficient shuffling was successfully done in this repetition
        let mut is_success = false;

        // Select random subset of commitments to be corrupt (do not corrupt indx 0)
        let bad_commitment_indices = HashSet::from_iter((1..VECTOR_LENGTH).choose_multiple(&mut rng, CORRUPTED_COMMITMENTS));

        // Initially all cups have 0 water apart for the one cup we care about tracking
        let mut water_cups: HashMap<usize, f64> = HashMap::new();
        let target_cup_indx = 0; // Just pick the first cup and track that
        water_cups.insert(target_cup_indx, 1.0);

        // Do all the shuffles
        for t in 0..MAX_SHUFFLES {
            if t % 50 == 0 {
                println!("\tRound {}", t);
            }

            // Each shuffler distributes the water to all the cups
            distribute_water(&mut water_cups, &bad_commitment_indices, &mut rng);

            // Check whether target commitment is hidden sufficiently well
            let max_water = water_cups.values().max_by(|a, b| a.total_cmp(b)).unwrap();
            if *max_water < TARGET_EPS {
                let successes = sum_succ_in_round.entry(t).or_insert(0.0);
                *successes += 1.0;

                if !is_success {
                    is_success = true;
                }
            }

            // Sanity check: no water was placed in bad, i.e. corupted or opened, cups
            for (index, water) in water_cups.iter() {
                if bad_commitment_indices.contains(index) {
                    assert_eq!(*water, 0.0);
                }
            }
        }
    }

    println!("\n\tSuccess probability after rounds");
    println!("\t----------");
    for t in 0..MAX_SHUFFLES {
        if t == 0 || t == MAX_SHUFFLES - 1  || sum_succ_in_round.get(&t).unwrap() != sum_succ_in_round.get(cmp::max(&0,&(&t-1))).unwrap() {
            // Probability that shuffling completes in each round
            println!("\t{} \t {}",t+1, *sum_succ_in_round.get(&t).unwrap() / NUMBER_OF_REPETITIONS as f64);
        }
    }
}

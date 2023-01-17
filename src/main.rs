use std::collections::HashSet;
use rand::{seq::IteratorRandom, thread_rng};
use rand::rngs::ThreadRng;
use hashbrown::HashMap;

const VECTOR_LENGTH: usize = 2_usize.pow(14);

// Size of one local shuffle
const SHUFFLE_SIZE: usize = 128;

// Upper bound on number of shuffles in protocol execution
const MAX_SHUFFLES: usize = 4000;

// Number of repetitions over which the average benchmark outcomes are computed
const NUMBER_OF_REPETITIONS: usize = 1000;

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

/// Return the first round where we managed to perfectly hide the cup
fn get_success_round(sum_succ_in_round: HashMap<usize, f64>) -> usize {
//    println!("\n\tSuccess probability after rounds");
//    println!("\t----------");

    for t in 0..MAX_SHUFFLES {
        // Success probability of current round
        let round_success = sum_succ_in_round.get(&t).unwrap();
        // Average success probability of previous round (should not underflow if t==0)
        // let prev_round_success = sum_succ_in_round.get(&t.saturating_sub(1)).unwrap();

        // if t == 0 || t == MAX_SHUFFLES - 1  || round_success != prev_round_success {
        // Probability that shuffling completes in each round
        //println!("\t{} \t {}",t+1, round_success / NUMBER_OF_REPETITIONS as f64);
        //}

        if round_success / NUMBER_OF_REPETITIONS as f64 == 1.0 {
            return t+1;
        }
    }

    return 0;
}

fn run_sim(fraction_corrupted_commitments: f64, corrupted_commitments: usize, target_eps: f64) {
    let mut rng = thread_rng();

    // Object for computing averages later on
    let mut sum_succ_in_round: HashMap<usize, f64> = HashMap::new();
    for t in 0..MAX_SHUFFLES { // Initialize hashmap with a default value of zero
        sum_succ_in_round.insert(t, 0.0);
    }

//    println!("\t{NUMBER_OF_REPETITIONS} Repetitions:");
//    println!("\t------------");

    for _ in 0..NUMBER_OF_REPETITIONS {
//        println!("\tRepetition {}/{}", repetition+1, NUMBER_OF_REPETITIONS);

        // Flag to be set, when sufficient shuffling was successfully done in this repetition
        let mut is_success = false;

        // Select random subset of commitments to be corrupt (do not corrupt indx 0)
        let bad_commitment_indices = HashSet::from_iter((1..VECTOR_LENGTH).choose_multiple(&mut rng, corrupted_commitments));

        // Initially all cups have 0 water apart for the one cup we care about tracking
        let mut water_cups: HashMap<usize, f64> = HashMap::new();
        let target_cup_indx = 0; // Just pick the first cup and track that
        water_cups.insert(target_cup_indx, 1.0);

        // Do all the shuffles
        for t in 0..MAX_SHUFFLES {
            // if t % 500 == 0 {
            //  println!("\tRound {}", t);
            //}

            // Each shuffler distributes the water to all the cups
            distribute_water(&mut water_cups, &bad_commitment_indices, &mut rng);

            // Check whether target commitment is hidden sufficiently well
            let max_water = water_cups.values().max_by(|a, b| a.total_cmp(b)).unwrap();
            if *max_water < target_eps {
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

    let successful_round = get_success_round(sum_succ_in_round);
    println!("Simulation parameters: [{VECTOR_LENGTH} {SHUFFLE_SIZE}] [{fraction_corrupted_commitments} {target_eps}]: {successful_round}");
}

fn main() {
    // Run simulations for corruption thresholds from 1% to 49%
    for p in 1..=49 {
        // Fraction of corrupted cards
        let fraction_corrupted_commitments: f64 = p as f64/100.0;
        let corrupted_commitments: usize = ((VECTOR_LENGTH as f64) * fraction_corrupted_commitments) as usize;

        // Target Water level
        let target_eps: f64 = 4.0 / (VECTOR_LENGTH as f64 * (1.0 - fraction_corrupted_commitments));

        run_sim(fraction_corrupted_commitments, corrupted_commitments, target_eps);
    }
}

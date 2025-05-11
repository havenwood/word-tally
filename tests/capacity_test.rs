use indexmap::IndexMap;
use std::sync::Arc;
use std::time::Instant;
use word_tally::{Options, Processing};

type TallyMap = IndexMap<String, usize>;

// Get the number of threads that would be used
fn get_thread_count() -> usize {
    rayon::current_num_threads()
}

// Calculate capacity per thread based on total capacity and thread count
fn calc_per_thread_capacity(total_capacity: usize) -> usize {
    let threads = rayon::current_num_threads();
    total_capacity / threads.max(1)
}

#[test]
fn test_capacity_allocation() {
    use word_tally::performance::TALLY_MAP_CAPACITY;

    // Create options with parallel processing
    let options = Arc::new(Options::default().with_processing(Processing::Parallel));
    let perf = options.performance();

    // Get the capacity values
    let base_capacity = perf.tally_map_capacity();
    println!("TallyMap capacity constant: {}", TALLY_MAP_CAPACITY);
    println!("Estimated tally map capacity: {}", base_capacity);

    // Test thread-local map capacity calculation
    let thread_local_capacity = perf.per_thread_tally_map_capacity();
    println!("Thread local map capacity: {}", thread_local_capacity);

    // Create maps with both capacities
    let main_map = TallyMap::with_capacity(TALLY_MAP_CAPACITY);
    let thread_local_map = TallyMap::with_capacity(thread_local_capacity);

    println!(
        "Main map capacity: {}, Thread-local map capacity: {}",
        main_map.capacity(),
        thread_local_map.capacity()
    );

    // Calculate ideal thread-local capacity
    let num_threads = get_thread_count();
    println!("Number of threads: {}", num_threads);
    let calculated_thread_capacity = calc_per_thread_capacity(TALLY_MAP_CAPACITY);
    println!(
        "Calculated per-thread capacity: {}",
        calculated_thread_capacity
    );

    // Verify thread_local_capacity matches calculated value
    assert_eq!(
        thread_local_capacity, calculated_thread_capacity,
        "Thread-local capacity should match calculated value"
    );

    // Now simulate map growth with different capacities
    println!("\n--- Simulating map growth ---");

    // Test with thread-local capacity
    let start_time = Instant::now();
    perform_word_counting_simulation(thread_local_capacity, "Thread-local capacity");
    let thread_local_elapsed = start_time.elapsed();

    // Compare with using full capacity
    let start_time = Instant::now();
    perform_word_counting_simulation(TALLY_MAP_CAPACITY, "Full capacity");
    let full_capacity_elapsed = start_time.elapsed();

    println!(
        "Thread-local capacity performance: {:?}",
        thread_local_elapsed
    );
    println!("Full capacity performance: {:?}", full_capacity_elapsed);

    // Using properly sized thread-local maps should be more efficient
    println!(
        "Efficiency ratio: {:.2}x",
        full_capacity_elapsed.as_micros() as f64 / thread_local_elapsed.as_micros() as f64
    );
}

// Simulate inserting many words into a map with different initial capacities
fn perform_word_counting_simulation(initial_capacity: usize, label: &str) {
    let mut map = TallyMap::with_capacity(initial_capacity);
    let word_count = 100_000; // Simulate a large number of words

    println!("{} initial capacity: {}", label, map.capacity());

    // Insert unique words (worst case for hash map growth)
    for i in 0..word_count {
        let word = format!("word_{}", i);
        map.insert(word, 1);

        // Log capacity at powers of 10
        if i == 10 || i == 100 || i == 1000 || i == 10_000 || i == 50_000 {
            println!(
                "{} after {} insertions: capacity={}",
                label,
                i,
                map.capacity()
            );
        }
    }

    println!("{} final capacity: {}", label, map.capacity());
}

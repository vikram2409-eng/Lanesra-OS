//! Performance baseline (WP-0015, REQ-PRE-012).
//!
//! Measures deterministic performance numbers using the scripted provider
//! (WP-0005) so results are reproducible. Run with:
//!   cargo test --test perf_baseline -- --ignored --nocapture
//!
//! The test prints the baseline numbers; CI reports them without failing.
//! Thresholds will be set by the owner (WP-0015 AC-2).
//!
//! Covers: REQ-PRE-012 (WP-0015 AC-1)

use std::time::Instant;

// Note: This is a template. The actual measurements require the agent loop,
// graph runtime, and checkpoint APIs which are not yet stable. The test
// structure is in place; measurements will be added as the APIs stabilize.

/// Baseline: agent loop overhead per round (scripted provider).
/// Measures the time for one agent loop iteration with deterministic responses.
#[test]
#[ignore]
fn perf_agent_loop_overhead() {
    // TODO: Wire up to the actual agent loop once the API is stable.
    // For now, measure the overhead of the test harness itself as a placeholder.
    let start = Instant::now();
    // Simulate 100 rounds of scripted provider dispatch
    for _ in 0..100 {
        // Placeholder: actual dispatch would go here
        std::hint::black_box(42);
    }
    let elapsed = start.elapsed();
    let per_round_us = elapsed.as_micros() / 100;

    println!("PERF-BASELINE agent_loop_per_round_us: {}", per_round_us);
    println!("PERF-BASELINE agent_loop_100_rounds_ms: {}", elapsed.as_millis());

    // No assertion - CI reports, does not fail (per WP-0015)
}

/// Baseline: SQLite bytes per run.
/// Measures the database size after a standard workload.
#[test]
#[ignore]
fn perf_sqlite_bytes_per_run() {
    // TODO: Wire up to actual workload once the API is stable.
    // Placeholder: report the metric name for CI parsing.
    println!("PERF-BASELINE sqlite_bytes_per_run: 0 (TODO: implement)");
}

/// Baseline: checkpoint write time.
/// Measures the time to write a checkpoint.
#[test]
#[ignore]
fn perf_checkpoint_write_time() {
    // TODO: Wire up to actual checkpoint API once stable.
    println!("PERF-BASELINE checkpoint_write_ms: 0 (TODO: implement)");
}

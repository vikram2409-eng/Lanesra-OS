//! TEMPORARY (WP-0003 AC-2): proves a failing test turns the `rust (tests)` check red.
//! Reverted in the next commit on this branch; must never reach `main`.

#[test]
fn ci_turns_red_on_a_failing_test() {
    let word = "ci";
    assert_eq!(word.len(), 3, "deliberate failure for WP-0003 AC-2");
}

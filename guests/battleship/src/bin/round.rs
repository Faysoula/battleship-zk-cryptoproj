use battleship_core::{RoundCommit, RoundInput};
use risc0_zkvm::guest::env;

fn main() {
    // Read the round input (current state + shot position)
    let RoundInput { mut state, shot } = env::read();

    // Create commitment to old state
    let old_state = state.commit();

    // Apply the shot and get the result
    let hit = state.apply_shot(shot);

    // Create commitment to new state
    let new_state = state.commit();

    // Write the proof to the journal
    env::commit(&RoundCommit {
        old_state,
        new_state,
        shot,
        hit,
    });
}
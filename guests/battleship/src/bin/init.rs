use battleship_core::GameState;
use risc0_zkvm::guest::env;

fn main() {
    // Read the initial game state from the host
    let state: GameState = env::read();

    // Validate the board setup
    if !state.check() {
        panic!("Invalid game state: ships overlap or are out of bounds");
    }

    // Commit the state hash to the journal
    env::commit(&state.commit());
}
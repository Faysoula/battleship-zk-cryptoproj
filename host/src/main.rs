mod board_display;
mod ship_placement;

use battleship_core::GameState;
use battleship_guests::{INIT_ELF, INIT_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use board_display::BoardDisplay;
use ship_placement::interactive_ship_placement;

fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║   ZERO-KNOWLEDGE BATTLESHIP - RISC Zero      ║");
    println!("║   Cryptographic Proof-Based Gameplay         ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    // Phase 1: Ship Placement
    println!("📍 PHASE 1: Place Your Ships");
    let state = interactive_ship_placement()?;

    // Phase 2: Generate ZK Proof of Valid Board
    println!("\n PHASE 2: Generating Zero-Knowledge Proof...");
    println!("   This proves your board is valid WITHOUT revealing positions!");
    
    let commitment = prove_board_initialization(&state)?;
    
    println!("\nZero-Knowledge Proof Generated!");
    println!("   Board Commitment: {:?}", commitment);
    println!("\n   Your opponent can verify your board is legal");
    println!("   WITHOUT learning where your ships are!\n");

    // Display the final board
    let display = BoardDisplay::new();
    display.display_own_board(&state);

    println!("\n✓ Single-player board setup complete!");
    println!("  Next: Implement multiplayer gameplay with ZK proofs\n");

    Ok(())
}

/// Generate a zero-knowledge proof that the board setup is valid
fn prove_board_initialization(state: &GameState) -> anyhow::Result<risc0_zkvm::sha::Digest> {
    println!("   Building execution environment...");
    let env = ExecutorEnv::builder()
        .write(state)?
        .build()?;

    println!("   Running ZK prover (this may take a moment in dev mode)...");
    let prover = default_prover();
    let prove_info = prover.prove(env, INIT_ELF)?;

    println!("   Verifying proof...");
    prove_info.receipt.verify(INIT_ID)?;

    let commitment: risc0_zkvm::sha::Digest = prove_info.receipt.journal.decode()?;
    
    println!("   ✓ Proof verified successfully!");

    Ok(commitment)
}
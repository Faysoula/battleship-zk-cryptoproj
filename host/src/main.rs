mod board_display;
mod game_coordinator;
mod network;
mod network_protocol;
mod ship_placement;

use battleship_core::GameState;
use battleship_guests::{INIT_ELF, INIT_ID};
use game_coordinator::GameCoordinator;
use network::NetworkConnection;
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::io::{self, Write};
use bytemuck::cast_slice;

fn main() -> anyhow::Result<()> {

    //this is just to check the ids before playing for testing remove later!!
    // println!("Method IDs:");
    // println!(" INIT_ID: {}", hex::encode(cast_slice(&battleship_guests::INIT_ID)));
    // println!(" ROUND_ID: {}", hex::encode(cast_slice(&battleship_guests::ROUND_ID)));


    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║   ZERO-KNOWLEDGE BATTLESHIP - Multiplayer     ║");
    println!("║   Network Play with Cryptographic Proofs      ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    // Choose mode
    println!("Choose mode:");
    println!("  1. Host a game (wait for opponent)");
    println!("  2. Join a game (connect to opponent)");
    print!("\nEnter choice (1/2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    let (network, starts_first) = match choice.trim() {
        "1" => {
            let network = NetworkConnection::host(7878)?;
            (network, true) // Host goes first
        }
        "2" => {
            print!("Enter opponent's IP address: ");
            io::stdout().flush()?;
            let mut ip = String::new();
            io::stdin().read_line(&mut ip)?;
            
            let network = NetworkConnection::connect(ip.trim(), 7878)?;
            (network, false) // Client goes second
        }
        _ => anyhow::bail!("Invalid choice"),
    };

    // Get player name
    print!("\nEnter your name: ");
    io::stdout().flush()?;
    let mut player_name = String::new();
    io::stdin().read_line(&mut player_name)?;
    let player_name = player_name.trim().to_string();

    // Ship placement
    println!("\nSHIP PLACEMENT");
    let state = ship_placement::interactive_ship_placement()?;

    // Generate ZK proof
    println!("\nGenerating board commitment proof...");
    let commitment = prove_board_init(&state)?;
    println!("Your Board Commitment: {:?}", commitment);

    // Start game
    let mut coordinator = GameCoordinator::new(
        state,
        commitment,
        network,
        player_name,
        starts_first,
    );

    coordinator.handshake()?;
    coordinator.play_game()?;

    println!("\nGame Over! Thanks for playing!\n");
    Ok(())
}

fn prove_board_init(state: &GameState) -> anyhow::Result<risc0_zkvm::sha::Digest> {
    let env = ExecutorEnv::builder().write(state)?.build()?;
    let prover = default_prover();
    let prove_info = prover.prove(env, INIT_ELF)?;
    prove_info.receipt.verify(INIT_ID)?;
    let commitment = prove_info.receipt.journal.decode()?;
    Ok(commitment)
}
use crate::board_display::BoardDisplay;
use crate::network::NetworkConnection;
use crate::network_protocol::{GameMessage, ProofData};
use battleship_core::{GameState, HitType, Position, RoundCommit, RoundInput};
use battleship_guests::{ROUND_ELF, ROUND_ID};
use risc0_zkvm::{default_prover, sha::Digest, ExecutorEnv};
use std::io::{self, Write};

pub struct GameCoordinator {
    my_state: GameState,
    my_commitment: Digest,
    my_display: BoardDisplay,
    
    opponent_commitment: Digest,
    opponent_display: BoardDisplay,
    
    network: NetworkConnection,
    player_name: String,
    opponent_name: String,
    is_my_turn: bool,
}

impl GameCoordinator {
    pub fn new(
        my_state: GameState,
        my_commitment: Digest,
        network: NetworkConnection,
        player_name: String,
        starts_first: bool,
    ) -> Self {
        Self {
            my_state,
            my_commitment,
            my_display: BoardDisplay::new(),
            opponent_commitment: Digest::default(),
            opponent_display: BoardDisplay::new(),
            network,
            player_name,
            opponent_name: String::new(),
            is_my_turn: starts_first,
        }
    }

    /// Exchange initial board commitments
    pub fn handshake(&mut self) -> anyhow::Result<()> {
        println!("\n🤝 Exchanging board commitments...");
        
        // Send our commitment
        self.network.send(&GameMessage::BoardReady {
            commitment: self.my_commitment,
            player_name: self.player_name.clone(),
        })?;
        
        // Receive opponent's commitment
        match self.network.receive()? {
            GameMessage::BoardReady { commitment, player_name } => {
                self.opponent_commitment = commitment;
                self.opponent_name = player_name.clone();
                println!("✓ Received commitment from {}", player_name);
                println!("   Opponent Commitment: {:?}", commitment);
            }
            _ => anyhow::bail!("Expected BoardReady message"),
        }
        
        println!("\n✓ Handshake complete! Game starting...\n");
        Ok(())
    }

    /// Main game loop
   pub fn play_game(&mut self) -> anyhow::Result<()> {
    loop {
        self.display_boards();
        
        if self.is_my_turn {
            // My turn: keep shooting until I miss
            loop {
                let hit_result = self.take_turn()?;
                
                // Check if I won
                if self.opponent_display.ships_remaining() == 0 {
                    println!("\n🎉 YOU WIN! All opponent ships destroyed!");
                    self.network.send(&GameMessage::GameOver {
                        winner: self.player_name.clone(),
                    })?;
                    return Ok(());
                }
                
                // If miss, switch turns
                match hit_result {
                    HitType::Miss => {
                        println!("\n⚠️  You missed! Turn passes to opponent.\n");
                        break;
                    }
                    HitType::Hit => {
                        println!("\n🔥 HIT! You get another shot!\n");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    HitType::Sunk(_) => {
                        println!("\n💥 SHIP SUNK! You get another shot!\n");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
            }
        } else {
            // Opponent's turn: they keep shooting until they miss
            loop {
                let hit_result = self.respond_to_shot()?;
                
                // Check if they won
                if self.my_display.ships_remaining() == 0 {
                    println!("\n💔 YOU LOSE! All your ships destroyed!");
                    return Ok(());
                }
                
                // If they missed, switch turns
                match hit_result {
                    HitType::Miss => {
                        println!("\n✅ Opponent missed! Your turn!\n");
                        break;
                    }
                    HitType::Hit => {
                        println!("\n⚠️  Opponent hit! They shoot again...\n");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    HitType::Sunk(_) => {
                        println!("\n💔 Opponent sunk a ship! They shoot again...\n");
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                }
            }
        }
        
        self.is_my_turn = !self.is_my_turn;
    }
}

    fn take_turn(&mut self) -> anyhow::Result<HitType> {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║           YOUR TURN                   ║");
    println!("╚═══════════════════════════════════════╝");
    
    let shot = self.prompt_shot()?;
    
    println!("\n🎯 Firing at {}...", shot);
    self.network.send(&GameMessage::TakeShot { position: shot })?;
    
    println!("⏳ Waiting for ZK proof from opponent...");
    match self.network.receive()? {
        GameMessage::ShotResult { position, hit_type, proof } => {
            println!("🔐 Received ZK proof, verifying...");
            
            // Verify the proof
            self.verify_shot_proof(position, &hit_type, &proof)?;
            
            // Record result
            self.opponent_display.record_shot(position, hit_type.clone());
            
            // Display result
            match &hit_type {
                HitType::Miss => println!("💨 MISS!"),
                HitType::Hit => println!("💥 HIT!"),
                HitType::Sunk(ship) => println!("🎯 SUNK {:?}!", ship),
            }
            
            Ok(hit_type)
        }
        GameMessage::GameOver { winner } => {
            println!("\n💔 {} wins!", winner);
            std::process::exit(0);
        }
        _ => anyhow::bail!("Unexpected message"),
    }
}

    fn respond_to_shot(&mut self) -> anyhow::Result<HitType> {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║        OPPONENT'S TURN                ║");
    println!("╚═══════════════════════════════════════╝");
    
    println!("⏳ Waiting for opponent's shot...");
    
    match self.network.receive()? {
        GameMessage::TakeShot { position } => {
            println!("🎯 Opponent shot at {}", position);
            println!("🔐 Generating ZK proof of result...");
            
            // Generate proof
            let (hit_type, proof) = self.generate_shot_proof(position)?;
            
            // Send result
            self.network.send(&GameMessage::ShotResult {
                position,
                hit_type: hit_type.clone(),
                proof,
            })?;
            
            // Record on our board
            self.my_display.record_shot(position, hit_type.clone());
            
            // Display result
            match &hit_type {
                HitType::Miss => println!("💨 They MISSED!"),
                HitType::Hit => println!("💥 They HIT your ship!"),
                HitType::Sunk(ship) => println!("🎯 They SUNK your {:?}!", ship),
            }
            
            Ok(hit_type)
        }
        GameMessage::GameOver { winner } => {
            println!("\n🎉 {} wins!", winner);
            std::process::exit(0);
        }
        _ => anyhow::bail!("Unexpected message"),
    }
}

    fn generate_shot_proof(&mut self, shot: Position) -> anyhow::Result<(HitType, ProofData)> {
        let input = RoundInput {
            state: self.my_state.clone(),
            shot,
        };
        
        let old_commit = self.my_state.commit();
        let hit_type = self.my_state.apply_shot(shot);
        let new_commit = self.my_state.commit();
        
        // Generate ZK proof
        let env = ExecutorEnv::builder().write(&input)?.build()?;
        let prover = default_prover();
        let prove_info = prover.prove(env, ROUND_ELF)?;
        
        let commit = RoundCommit {
            old_state: old_commit,
            new_state: new_commit,
            shot,
            hit: hit_type.clone(),
        };
        
        let proof = ProofData::from_receipt(prove_info.receipt, commit)?;
        
        Ok((hit_type, proof))
    }

    fn verify_shot_proof(
        &self,
        position: Position,
        hit_type: &HitType,
        proof: &ProofData,
    ) -> anyhow::Result<()> {
        let receipt = proof.to_receipt()?;
        
        // Verify the ZK proof
        receipt.verify(ROUND_ID)?;
        
        // Decode and verify the commitment
        let commit: RoundCommit = receipt.journal.decode()?;
        
        // Verify claims
        if commit.old_state != self.opponent_commitment {
            anyhow::bail!("Proof uses wrong state commitment!");
        }
        if commit.shot != position {
            anyhow::bail!("Proof is for wrong shot position!");
        }
        if &commit.hit != hit_type {
            anyhow::bail!("Proof hit type doesn't match!");
        }
        
        // Update opponent commitment for next round
        // (We'd need to make opponent_commitment mutable)
        
        println!("✅ ZK Proof verified! Result is cryptographically proven.");
        Ok(())
    }

    fn prompt_shot(&self) -> anyhow::Result<Position> {
        loop {
            print!("Enter coordinates to fire (x,y): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            let parts: Vec<&str> = input.trim().split(',').collect();
            if parts.len() != 2 {
                println!("Invalid format. Use: x,y");
                continue;
            }
            
            let x: u32 = match parts[0].trim().parse() {
                Ok(v) if v < 10 => v,
                _ => {
                    println!("X must be 0-9");
                    continue;
                }
            };
            
            let y: u32 = match parts[1].trim().parse() {
                Ok(v) if v < 10 => v,
                _ => {
                    println!("Y must be 0-9");
                    continue;
                }
            };
            
            return Ok(Position::new(x, y));
        }
    }

    fn display_boards(&self) {
        println!("\n");
        println!("╔═══════════════════════════════════════════════╗");
        println!("║  {} vs {}                    ", self.player_name, self.opponent_name);
        println!("║  Your Ships: {} | Opponent Ships: {}         ", 
                 self.my_display.ships_remaining(),
                 self.opponent_display.ships_remaining());
        println!("╚═══════════════════════════════════════════════╝");
        
        self.opponent_display.display_opponent_board();
        self.my_display.display_own_board(&self.my_state);
    }
}
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

    pub fn handshake(&mut self) -> anyhow::Result<()> {
        println!("\n🤝 Exchanging board commitments...");
        
        self.network.send(&GameMessage::BoardReady {
            commitment: self.my_commitment,
            player_name: self.player_name.clone(),
        })?;
        
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

    pub fn play_game(&mut self) -> anyhow::Result<()> {
        loop {
            if self.is_my_turn {
                
                // ✅ Show both boards ONLY at start of your turn
                self.display_boards();

                loop {
                    let hit_result = self.take_turn()?;

                    // ✅ After each shot, show ONLY opponent board
                    self.display_opponent_board_after_shot(&hit_result);

                    if self.opponent_display.ships_remaining() == 0 {
                        println!("\n*** YOU WIN! All opponent ships destroyed! ***");
                        self.network.send(&GameMessage::GameOver {
                            winner: self.player_name.clone(),
                        })?;
                        return Ok(());
                    }

                    match hit_result {
                        HitType::Miss => {
                            println!("\nYou missed! Turn passes to opponent.\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            break;
                        }
                        HitType::Hit => {
                            println!("\nHIT! You get another shot!\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            // ✅ Keep looping WITHOUT refreshing both boards
                        }
                        HitType::Sunk(_) => {
                            println!("\nSHIP SUNK! You get another shot!\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                    }
                }
            } else {
                // ✅ Opponent turn still shows both boards normally
                loop {
                    self.display_boards();
                    
                    let hit_result = self.respond_to_shot()?;

                    self.display_boards_after_opponent_shot(&hit_result);

                    if self.my_display.ships_remaining() == 0 {
                        println!("\n*** YOU LOSE! All your ships destroyed! ***");
                        return Ok(());
                    }

                    match hit_result {
                        HitType::Miss => {
                            println!("\nOpponent missed! Your turn!\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            break;
                        }
                        HitType::Hit => {
                            println!("\nOpponent hit! They shoot again...\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                        HitType::Sunk(_) => {
                            println!("\nOpponent sunk a ship! They shoot again...\n");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                        }
                    }
                }
            }
            
            self.is_my_turn = !self.is_my_turn;
        }
    }

    fn display_boards(&self) {
        print!("\x1B[2J\x1B[1;1H");
        
        println!("\n");
        println!("╔═══════════════════════════════════════════════╗");
        println!("║  {} vs {}                    ", self.player_name, self.opponent_name);
        println!("║  Your Ships: {} | Opponent Ships: {}            ", 
                 self.my_display.ships_remaining(),
                 self.opponent_display.ships_remaining());
        if self.is_my_turn {
            println!("║  >>> YOUR TURN <<<                            ║");
        } else {
            println!("║  >>> OPPONENT'S TURN <<<                      ║");
        }
        println!("╚═══════════════════════════════════════════════╝");
        
        self.opponent_display.display_opponent_board();
        self.my_display.display_own_board(&self.my_state);
    }

    fn display_opponent_board_after_shot(&self, hit_type: &HitType) {
        print!("\x1B[2J\x1B[1;1H");
        
        println!("\n");
        println!("╔═══════════════════════════════════════════════╗");
        println!("║  SHOT RESULT                                  ║");
        match hit_type {
            HitType::Miss => println!("║  MISS!                                        ║"),
            HitType::Hit => println!("║  HIT!                                         ║"),
            HitType::Sunk(ship) => println!("║  SUNK {:?}!                              ║", ship),
        }
        println!("╚═══════════════════════════════════════════════╝");
        
        println!("\nOPPONENT'S BOARD (Updated):");
        self.opponent_display.display_opponent_board();

        // ✅ NO self-board here — this was the bug
    }

    fn display_boards_after_opponent_shot(&self, hit_type: &HitType) {
        print!("\x1B[2J\x1B[1;1H");
        
        println!("\n");
        println!("╔═══════════════════════════════════════════════╗");
        println!("║  OPPONENT'S SHOT RESULT                       ║");
        match hit_type {
            HitType::Miss => println!("║  They MISSED!                                 ║"),
            HitType::Hit => println!("║  They HIT your ship!                          ║"),
            HitType::Sunk(ship) => println!("║  They SUNK your {:?}!                    ║", ship),
        }
        println!("╚═══════════════════════════════════════════════╝");
        
        println!("\nOPPONENT'S BOARD:");
        self.opponent_display.display_opponent_board();
        
        println!("\nYOUR BOARD (Updated with damage):");
        self.my_display.display_own_board(&self.my_state);
    }

    fn take_turn(&mut self) -> anyhow::Result<HitType> {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║        TAKE YOUR SHOT                 ║");
        println!("╚═══════════════════════════════════════╝");
        
        let shot = self.prompt_shot()?;
        
        println!("\nFiring at {}...", shot);
        self.network.send(&GameMessage::TakeShot { position: shot })?;
        
        println!("⏳ Waiting for ZK proof from opponent...");
        match self.network.receive()? {
            GameMessage::ShotResult { position, hit_type, proof } => {
                println!("🔐 Verifying ZK proof...");
                
                self.verify_shot_proof(position, &hit_type, &proof)?;
                self.opponent_display.record_shot(position, hit_type.clone());
                
                println!("✅ Proof verified!");
                
                Ok(hit_type)
            }
            GameMessage::GameOver { winner } => {
                println!("\n{} wins!", winner);
                std::process::exit(0);
            }
            _ => anyhow::bail!("Unexpected message"),
        }
    }

    fn respond_to_shot(&mut self) -> anyhow::Result<HitType> {
        println!("\nWaiting for opponent's shot...");
        
        match self.network.receive()? {
            GameMessage::TakeShot { position } => {
                println!("Opponent shot at {}", position);
                println!("🔐 Generating ZK proof of result...");
                
                let (hit_type, proof) = self.generate_shot_proof(position)?;
                
                self.network.send(&GameMessage::ShotResult {
                    position,
                    hit_type: hit_type.clone(),
                    proof,
                })?;
                
                self.my_display.record_shot(position, hit_type.clone());
                
                println!("✅ Proof sent!");
                
                Ok(hit_type)
            }
            GameMessage::GameOver { winner } => {
                println!("\n{} wins!", winner);
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
        
        let old_commit = self.my_commitment;
        let hit_type = self.my_state.apply_shot(shot);
        let new_commit = self.my_state.commit();
        
        self.my_commitment = new_commit;
        
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
        &mut self,
        position: Position,
        hit_type: &HitType,
        proof: &ProofData,
    ) -> anyhow::Result<()> {
        let receipt = proof.to_receipt()?;
        
        receipt.verify(ROUND_ID)?;
        
        let commit: RoundCommit = receipt.journal.decode()?;
        
        if commit.old_state != self.opponent_commitment {
            anyhow::bail!("Proof uses wrong state commitment!");
        }
        if commit.shot != position {
            anyhow::bail!("Proof is for wrong shot position!");
        }
        if &commit.hit != hit_type {
            anyhow::bail!("Proof hit type doesn't match!");
        }
        
        self.opponent_commitment = commit.new_state;
        
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
}

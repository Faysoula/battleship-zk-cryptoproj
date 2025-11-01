use battleship_core::{GameState, HitType, Position, ShipClass, BOARD_SIZE};
use std::collections::HashSet;

pub struct BoardDisplay {
    shots: HashSet<Position>,
    hits: HashSet<Position>,
    sunk_ships: HashSet<ShipClass>,
}

impl BoardDisplay {
    pub fn new() -> Self {
        Self {
            shots: HashSet::new(),
            hits: HashSet::new(),
            sunk_ships: HashSet::new(),
        }
    }

    pub fn record_shot(&mut self, pos: Position, result: HitType) {
        self.shots.insert(pos);
        match result {
            HitType::Hit => {
                self.hits.insert(pos);
            }
            HitType::Sunk(ship_class) => {
                self.hits.insert(pos);
                self.sunk_ships.insert(ship_class);
            }
            HitType::Miss => {}
        }
    }

    /// Display your own board (shows ships)
    pub fn display_own_board(&self, state: &GameState) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║        YOUR BOARD (Ships Visible)     ║");
        println!("╠═══════════════════════════════════════╣");
        
        // Header
        print!("║   ");
        for x in 0..BOARD_SIZE {
            print!(" {} ", x);
        }
        println!(" ║");
        println!("║  ┌────────────────────────────────┐ ║");

        // Board rows
        for y in 0..BOARD_SIZE {
            print!("║ {} │", y);
            for x in 0..BOARD_SIZE {
                let pos = Position::new(x as u32, y as u32);
                let symbol = self.get_own_board_symbol(pos, state);
                print!(" {} ", symbol);
            }
            println!("│ ║");
        }

        println!("║  └────────────────────────────────┘ ║");
        println!("╚═══════════════════════════════════════╝");
        
        // Legend
        println!("\n  Legend: [A]=Carrier [B]=Battleship [C]=Cruiser [S]=Sub [D]=Destroyer");
        println!("          [X]=Hit  [O]=Miss  [~]=Water");
    }

    /// Display opponent's board (ships hidden, only shows hits/misses)
    pub fn display_opponent_board(&self) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║    OPPONENT BOARD (Ships Hidden)      ║");
        println!("╠═══════════════════════════════════════╣");
        
        // Header
        print!("║   ");
        for x in 0..BOARD_SIZE {
            print!(" {} ", x);
        }
        println!(" ║");
        println!("║  ┌────────────────────────────────┐ ║");

        // Board rows
        for y in 0..BOARD_SIZE {
            print!("║ {} │", y);
            for x in 0..BOARD_SIZE {
                let pos = Position::new(x as u32, y as u32);
                let symbol = if self.hits.contains(&pos) {
                    "X" // Hit (proven by ZK proof)
                } else if self.shots.contains(&pos) {
                    "O" // Miss (proven by ZK proof)
                } else {
                    "~" // Unknown
                };
                print!(" {} ", symbol);
            }
            println!("│ ║");
        }

        println!("║  └────────────────────────────────┘ ║");
        println!("╚═══════════════════════════════════════╝");
        
        // Ships sunk
        if !self.sunk_ships.is_empty() {
            print!("\n  Ships Sunk: ");
            for ship in &self.sunk_ships {
                print!("{:?} ", ship);
            }
            println!();
        }
        
        println!("\n  [X]=Hit (ZK Verified)  [O]=Miss (ZK Verified)  [~]=Unknown");
    }

    fn get_own_board_symbol(&self, pos: Position, state: &GameState) -> &str {
        // Check if there's a ship at this position
        for ship in &state.ships {
            if ship.points().any(|p| p == pos) {
                // Check if this position was hit
                if self.hits.contains(&pos) {
                    return "X"; // Hit on your ship
                }
                // Return ship symbol
                return match ship.class {
                    ShipClass::Carrier => "A",
                    ShipClass::Battleship => "B",
                    ShipClass::Cruiser => "C",
                    ShipClass::Submarine => "S",
                    ShipClass::Destroyer => "D",
                };
            }
        }

        // No ship, check if shot
        if self.shots.contains(&pos) {
            "O" // Miss
        } else {
            "~" // Water
        }
    }

    pub fn ships_remaining(&self) -> usize {
        ShipClass::list().len() - self.sunk_ships.len()
    }
}
use battleship_core::{Direction, GameState, Position, Ship, ShipClass, BOARD_SIZE};
use std::io::{self, Write};

pub fn interactive_ship_placement() -> anyhow::Result<GameState> {
    println!("\n╔═══════════════════════════════════════════════╗");
    println!("║       SHIP PLACEMENT - Zero-Knowledge         ║");
    println!("║  Your board will be cryptographically         ║");
    println!("║  committed using RISC Zero proofs!            ║");
    println!("╚═══════════════════════════════════════════════╝\n");

    println!("Choose placement method:");
    println!("  1. Manual placement (choose each ship position)");
    println!("  2. Random placement (quick setup for testing or if youre lazy)");
    print!("\nEnter choice (1/2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;

    match choice.trim() {
        "1" => manual_placement(),
        "2" => random_placement(),
        _ => {
            println!("Invalid choice, using random placement");
            random_placement()
        }
    }
}

fn random_placement() -> anyhow::Result<GameState> {
    println!("\nGenerating random ship placement...");
    let state: GameState = rand::random();
    
    display_board(&state);
    println!("\nShips randomly placed!");
    println!("   Press Enter to continue...");
    
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    
    Ok(state)
}

fn manual_placement() -> anyhow::Result<GameState> {
    let mut state = GameState::new(rand::random());

    let ships_to_place = [
        (ShipClass::Carrier, "Carrier", 5),
        (ShipClass::Battleship, "Battleship", 4),
        (ShipClass::Cruiser, "Cruiser", 3),
        (ShipClass::Submarine, "Submarine", 3),
        (ShipClass::Destroyer, "Destroyer", 2),
    ];

    for (ship_class, name, length) in ships_to_place {
        loop {
            display_board(&state);
            println!("\n┌─────────────────────────────────────┐");
            println!("│ Placing: {} (length: {})        ", name, length);
            println!("└─────────────────────────────────────┘");

            // Get position
            let pos = match prompt_position("Enter starting position (x,y): ")? {
                Some(p) => p,
                None => continue,
            };

            // Get direction
            let dir = match prompt_direction()? {
                Some(d) => d,
                None => continue,
            };

            // Try to place the ship
            let ship = Ship::new(ship_class, pos, dir);
            if state.add_ship(ship) {
                println!("✓ {} placed successfully!", name);
                break;
            } else {
                println!("  Invalid placement! Ship overlaps or goes out of bounds.");
                println!("  Press Enter to try again...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
            }
        }
    }

    display_board(&state);
    println!("\n  All ships placed! Board is ready for ZK commitment.\n");

    Ok(state)
}

fn display_board(state: &GameState) {
    println!("\n  ┌────────────────────────────────┐");
    print!("  │ ");
    for x in 0..BOARD_SIZE {
        print!(" {} ", x);
    }
    println!(" │");
    println!("  ├────────────────────────────────┤");

    for y in 0..BOARD_SIZE {
        print!("{} │ ", y);
        for x in 0..BOARD_SIZE {
            let pos = Position::new(x as u32, y as u32);
            let mut found = false;
            
            for ship in &state.ships {
                if ship.points().any(|p| p == pos) {
                    let symbol = match ship.class {
                        ShipClass::Carrier => "A",
                        ShipClass::Battleship => "B",
                        ShipClass::Cruiser => "C",
                        ShipClass::Submarine => "S",
                        ShipClass::Destroyer => "D",
                    };
                    print!(" {} ", symbol);
                    found = true;
                    break;
                }
            }
            
            if !found {
                print!(" ~ ");
            }
        }
        println!(" │");
    }

    println!("  └────────────────────────────────┘");
}

fn prompt_position(prompt: &str) -> anyhow::Result<Option<Position>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    // Parse "x,y" format
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 2 {
        println!("Invalid format. Use: x,y (e.g., 3,5)");
        return Ok(None);
    }

    let x: u32 = match parts[0].trim().parse() {
        Ok(v) if v < BOARD_SIZE as u32 => v,
        _ => {
            println!("X must be between 0 and {}", BOARD_SIZE - 1);
            return Ok(None);
        }
    };

    let y: u32 = match parts[1].trim().parse() {
        Ok(v) if v < BOARD_SIZE as u32 => v,
        _ => {
            println!("Y must be between 0 and {}", BOARD_SIZE - 1);
            return Ok(None);
        }
    };

    Ok(Some(Position::new(x, y)))
}

fn prompt_direction() -> anyhow::Result<Option<Direction>> {
    print!("Enter direction (h=horizontal, v=vertical): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim().to_lowercase().as_str() {
        "h" | "horizontal" => Ok(Some(Direction::Horizontal)),
        "v" | "vertical" => Ok(Some(Direction::Vertical)),
        _ => {
            println!("Invalid direction. Use 'h' or 'v'");
            Ok(None)
        }
    }
}
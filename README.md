# Zero-Knowledge Battleship

A multiplayer Battleship game using RISC Zero zero-knowledge proofs to ensure fair gameplay without revealing ship positions.

## Overview

This implementation uses RISC Zero zkVM to generate cryptographic proofs for each move. Players can verify that their opponent's responses (hit/miss) are correct without seeing the opponent's board layout.

## Prerequisites

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```
rustc --version
```

### 2. Install RISC Zero Toolchain
```bash
curl -L https://risczero.com/install | bash
source ~/.bashrc
rzup install rust
```

Verify installation:
```
rzup --version
```

### 3. Install Build Dependencies (Linux/WSL)
```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev git -y
```

## Installation

### Clone the Repository
```bash
git clone https://github.com/Faysoula/battleship-zk-cryptoproj.git
cd battleship-zk-cryptoproj
```

### Build the Project

```
cargo build --release
```

First build takes 5-10 minutes.

If the build fails, it is usually due to one of the following:

1.  **The C++ compiler (used by some dependencies) crashed near the end of the build.**
    This is rare, but it does happen occasionally. Simply running the build again often fixes it.

2.  **The RISC Zero dependency download failed verification.**
    This typically happens due to an unstable or inconsistent network connection.
    Try rebuilding while connected to a stable network to ensure the dependency checks pass.

## Network Setup with Tailscale

For playing over the internet, both players need Tailscale.

### Install Tailscale

Both players run:
```bash
curl -fsSL https://tailscale.com/install.sh | sh
sudo tailscale up
```

### Important: Use Same Tailscale Account

CRITICAL: Both players must authenticate with the SAME Tailscale account for the connection to work.

1. One player creates a Tailscale account
2. Both players use that same account when running `sudo tailscale up`
3. Open the authentication URL in browser
4. Sign in with the shared account

### Get Your Tailscale IP
```
tailscale ip -4
```

Example output: `100.64.1.5`

Share this IP with your friend.

## Running the Game

### Host Player (Player 1)
```bash
cd battleship-zk-cryptoproj
cargo run --release
```

1. Choose option `1` (Host a game)
2. Enter your name
3. Choose ship placement:
   - Option `1`: Manual placement
   - Option `2`: Random placement
4. Wait for opponent to connect

### Joining Player (Player 2)
```bash
cd battleship-zk-cryptoproj
cargo run --release
```

1. Choose option `2` (Join a game)
2. Enter host's Tailscale IP (e.g., `100.64.1.5`)
3. Enter your name
4. Place your ships

### Gameplay

- Enter coordinates as: `x,y` (e.g., `3,5`)
- Board coordinates range from 0-9
- After a hit, you get another shot
- After a miss, turn switches to opponent
- Zero-knowledge proofs are generated and verified for each move
- Game ends when all of one player's ships are destroyed

## Troubleshooting

### Build Errors

Missing C compiler:
```bash
sudo apt install build-essential -y
```

Clean rebuild:
```bash
cargo clean
RISC0_DEV_MODE=1 cargo build --release
```

### Proof Verification Failures

If you get "claim digest does not match" errors, ensure:

1. Both players have identical code (same git commit)
2. Both players built with `RISC0_DEV_MODE=1`
3. Both have same RISC Zero toolchain version:
```bash
   rzup --version
   rustc --version
```

If versions differ, sync to same version:
```bash
rzup update
cargo clean
RISC0_DEV_MODE=1 cargo build --release
```

## Project Structure
```
.
├── Cargo.toml              # Workspace configuration
├── rust-toolchain.toml     # Rust toolchain specification
├── build_guests.sh         # Guest build script
├── .gitignore
├── .gitattributes
│
├── core/                   # Shared game logic
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # Game state, ships, and core types
│
├── guests/                 # RISC Zero guest programs (ZK circuits)
│   ├── Cargo.toml
│   ├── build.rs            # Guest build configuration
│   ├── src/
│   │   └── lib.rs
│   └── battleship/
│       ├── Cargo.toml
│       └── src/
│           ├── bin/
│           │   └── init.rs # Board initialization proof
│           ├── init.rs
│           └── round.rs    # Round execution proof
│
├── host/                   # Main application
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 # Entry point
│       ├── board_display.rs        # UI rendering
│       ├── game_coordinator.rs     # Game loop and state
│       ├── network.rs              # TCP networking
│       ├── network_protocol.rs     # Message protocol
│       └── ship_placement.rs       # Ship setup UI
│
├── prebuilt/               # Precompiled guest binaries (optional)
│   ├── init.bin
│   ├── round.bin
│   └── methods.rs
│
└── src/
    └── lib.rs              # Workspace library root
```

## How Zero-Knowledge Proofs Work

1. Board Setup: Each player generates a cryptographic commitment to their board layout
2. Each Shot: The defending player generates a ZK proof showing whether the shot was a hit or miss
3. Verification: The attacking player verifies the proof without learning ship positions
4. Security: Cheating is cryptographically impossible - all moves are proven correct

## Development vs Production

### Development Mode
```bash
RISC0_DEV_MODE=1 cargo run
```

- Fast builds and execution
- Proofs are generated but not cryptographically secure
- Perfect for testing and development

### Production Mode
```bash
cargo run --release
```

- Generates real zero-knowledge proofs
- slightly slower
- Cryptographically secure

## Testing

Run unit tests:
```bash
RISC0_DEV_MODE=1 cargo test
```

## License

Apache License 2.0

## Requirements

- Rust 1.70+
- RISC Zero toolchain
- Linux, macOS, or Windows with WSL2

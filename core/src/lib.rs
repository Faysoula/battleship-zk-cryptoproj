use serde::{Deserialize, Serialize};
use std::fmt::Display;
use risc0_zkvm::sha::{Digest, Sha256};

#[cfg(feature = "rand")]
use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    Rng,
};

pub const NUM_SHIPS: usize = 5;
pub const BOARD_SIZE: usize = 10;

// ============================================================================
// Basic Types
// ============================================================================

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Hash)]
pub enum ShipClass {
    Carrier,
    Battleship,
    Cruiser,
    Submarine,
    Destroyer,
}

impl ShipClass {
    /// Get the span (length) of the ship
    pub fn span(&self) -> u32 {
        match self {
            ShipClass::Carrier => 5,
            ShipClass::Battleship => 4,
            ShipClass::Cruiser => 3,
            ShipClass::Submarine => 3,
            ShipClass::Destroyer => 2,
        }
    }

    /// Get the sunk mask for the ship (e.g., for a span of 3, this is 0b000)
    pub fn sunk_mask(&self) -> u8 {
        (1u8 << self.span()) - 1
    }

    pub const fn list() -> &'static [ShipClass] {
        &[
            Self::Carrier,
            Self::Battleship,
            Self::Cruiser,
            Self::Submarine,
            Self::Destroyer,
        ]
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Step in a direction by a certain distance
    pub fn step(self, dir: Direction, dist: u32) -> Self {
        match dir {
            Direction::Vertical => Self {
                x: self.x,
                y: self.y + dist,
            },
            Direction::Horizontal => Self {
                x: self.x + dist,
                y: self.y,
            },
        }
    }

    pub fn in_bounds(&self) -> bool {
        self.x < BOARD_SIZE as u32 && self.y < BOARD_SIZE as u32
    }
}

impl From<(u32, u32)> for Position {
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
}

// Implement Display for Position
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    pub fn flip(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

// Random distribution for Direction
#[cfg(feature = "rand")]
impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        if rng.gen::<bool>() {
            Direction::Horizontal
        } else {
            Direction::Vertical
        }
    }
}

// ============================================================================
// Ship Structure
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Ship {
    pub class: ShipClass,
    pub pos: Position,
    pub dir: Direction,
    pub hit_mask: u8, // Each bit represents whether that segment has been hit
}

impl Ship {
    pub fn new(class: ShipClass, pos: impl Into<Position>, dir: Direction) -> Self {
        Ship {
            class,
            pos: pos.into(),
            dir,
            hit_mask: 0,
        }
    }

    // Create a new ship with a specified hit mask
    pub fn with_hit_mask(self, hit_mask: u8) -> Self {
        Self { hit_mask, ..self }
    }

    // Get an iterator over all points occupied by the ship
    pub fn points(&self) -> impl Iterator<Item = Position> + '_ {
        (0..self.class.span()).map(|offset| self.pos.step(self.dir, offset))
    }

    // Does this ship intersect with another ship?
    pub fn intersects(&self, other: &Self) -> bool {
        self.points().any(|p| other.points().any(|q| p == q))
    }

    pub fn in_bounds(&self) -> bool {
        self.pos.in_bounds() && self.pos.step(self.dir, self.class.span() - 1).in_bounds()
    }

    pub fn apply_shot(&mut self, shot: Position) -> HitType {
        let hit_index = self.points().position(|pos| pos == shot);
        match hit_index {
            Some(hit_index) => {
                self.hit_mask |= 1 << hit_index;
                if self.hit_mask == self.class.sunk_mask() {
                    HitType::Sunk(self.class)
                } else {
                    HitType::Hit
                }
            }
            None => HitType::Miss,
        }
    }
}

// ============================================================================
// Game State
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GameState {
    pub ships: Vec<Ship>,
    pub pepper: [u8; 16],
}

impl GameState {
    pub fn new(pepper: [u8; 16]) -> Self {
        Self {
            ships: Vec::new(),
            pepper, // random nonce to salt the state
        }
    }

    pub fn check(&self) -> bool {
        // Check all ships are in bounds
        for ship in &self.ships {
            if !ship.in_bounds() {
                return false;
            }
        }

        // Check each ship class appears exactly once
        let mut classes = ShipClass::list().to_vec();
        for ship in &self.ships {
            if let Some(pos) = classes.iter().position(|&c| c == ship.class) {
                classes.swap_remove(pos);
            } else {
                return false;
            }
        }
        if !classes.is_empty() {
            return false;
        }

        // Check no ships overlap
        for (i, ship_i) in self.ships.iter().enumerate() {
            for ship_j in self.ships.iter().skip(i + 1) {
                if ship_i.intersects(ship_j) {
                    return false;
                }
            }
        }

        true
    }

    pub fn add_ship(&mut self, new_ship: Ship) -> bool {
        if !new_ship.in_bounds() {
            return false;
        }

        for ship in &self.ships {
            if ship.class == new_ship.class || ship.intersects(&new_ship) {
                return false;
            }
        }

        self.ships.push(new_ship);
        true
    }

    // Apply a shot to the game state, returning the HitType
    pub fn apply_shot(&mut self, shot: Position) -> HitType {
        for ship in &mut self.ships {
            let hit = ship.apply_shot(shot);
            if matches!(hit, HitType::Hit | HitType::Sunk(_)) {
                return hit;
            }
        }
        HitType::Miss
    }

    // Compute the commitment digest of the game state THIS IS VERY IMPORTANT
    pub fn commit(&self) -> Digest {
        let bytes = bincode::serialize(self).expect("serialization should succeed");
        *risc0_zkvm::sha::Impl::hash_bytes(&bytes)
    }
}

#[cfg(feature = "rand")]
impl Distribution<GameState> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GameState {
        let mut positions: Vec<Position> = (0..BOARD_SIZE)
            .flat_map(|x| (0..BOARD_SIZE).map(move |y| Position::new(x as u32, y as u32)))
            .collect();
        positions.shuffle(rng);

        let mut state = GameState::new(rng.gen());
        
        'outer: for &ship_class in ShipClass::list() {
            for &pos in &positions {
                for dir in [Direction::Horizontal, Direction::Vertical] {
                    if state.add_ship(Ship::new(ship_class, pos, dir)) {
                        continue 'outer;
                    }
                }
            }
            panic!("Failed to place {:?}", ship_class);
        }

        assert!(state.check());
        state
    }
}

// ============================================================================
// Zero-Knowledge Types
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum HitType {
    Miss,
    Hit,
    Sunk(ShipClass),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RoundInput {
    pub state: GameState,
    pub shot: Position,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RoundCommit {
    pub old_state: Digest,
    pub new_state: Digest,
    pub shot: Position,
    pub hit: HitType,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_board() {
        let state = GameState {
            ships: vec![
                Ship::new(ShipClass::Carrier, (2, 3), Direction::Vertical),
                Ship::new(ShipClass::Battleship, (3, 1), Direction::Horizontal),
                Ship::new(ShipClass::Cruiser, (4, 7), Direction::Vertical),
                Ship::new(ShipClass::Submarine, (7, 5), Direction::Horizontal),
                Ship::new(ShipClass::Destroyer, (7, 7), Direction::Horizontal),
            ],
            pepper: [0; 16],
        };
        assert!(state.check());
    }

    #[test]
    #[cfg(feature = "rand")]
    fn test_random_boards() {
        for _ in 0..100 {
            let state: GameState = rand::random();
            assert!(state.check());
        }
    }
}
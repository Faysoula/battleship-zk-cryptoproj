use battleship_core::{HitType, Position, RoundCommit};
use risc0_zkvm::{Receipt, sha::Digest};
use serde::{Deserialize, Serialize};

/// Messages sent between players over the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    /// Initial handshake: send board commitment
    BoardReady {
        commitment: Digest,
        player_name: String,
    },

    /// Request to take a shot
    TakeShot {
        position: Position,
    },

    /// Response with ZK proof of hit/miss
    ShotResult {
        position: Position,
        hit_type: HitType,
        proof: ProofData,
    },

    /// Game over notification
    GameOver {
        winner: String,
    },

    /// Error message
    Error {
        message: String,
    },
}

/// Serializable proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    pub receipt_bytes: Vec<u8>,
    pub commit: RoundCommit,
}

impl ProofData {
    pub fn from_receipt(receipt: Receipt, commit: RoundCommit) -> anyhow::Result<Self> {
        let receipt_bytes = bincode::serialize(&receipt)?;
        Ok(Self {
            receipt_bytes,
            commit,
        })
    }

    pub fn to_receipt(&self) -> anyhow::Result<Receipt> {
        Ok(bincode::deserialize(&self.receipt_bytes)?)
    }
}
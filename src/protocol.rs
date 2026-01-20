use serde::{Deserialize, Serialize};

/// The type of frame traveling through the tunnel.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum FrameType {
    /// Standard encrypted transport frame containing IP packet.
    Transport,
    /// Keep-alive packet (Chaff) to maintain NAT mappings.
    Heartbeat,
    /// Fake Handshake (Obfuscation) to look like TLS.
    Handshake,
    /// Reliability Acknowledgment.
    Ack,
}

/// The headers for our Ghost Protocol (Wire Format).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameHeader {
    /// Monotonic sequence number.
    /// Used for:
    /// 1. Reordering (if we implement a reorder buffer later).
    /// 2. Basic replay protection.
    /// 3. Jitter calcluation (diff between send/recv times).
    pub seq: u64,
    /// The sequence number this frame acknowledges.
    /// For Data frames, this piggybacks the last seen seq (optional opt).
    /// For Ack frames, this is the payload.
    pub ack_num: u64,
    /// The type of payload.
    pub frame_type: FrameType,
}

/// The Atomic Unit of the Ghost Protocol.
/// This matches the MTU size + Overhead.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WireFrame {
    pub header: FrameHeader,
    /// Encrypted payload (Poly1305 tag included).
    pub payload: Vec<u8>,
}

impl WireFrame {
    /// Create a new data frame ready for the wire.
    pub fn new_data(seq: u64, payload: Vec<u8>) -> Self {
        Self {
            header: FrameHeader {
                seq,
                ack_num: 0, // Piggybacking not implemented yet
                frame_type: FrameType::Transport,
            },
            payload,
        }
    }

    /// Create an ACK frame.
    pub fn new_ack(seq: u64, ack_num: u64) -> Self {
        Self {
            header: FrameHeader {
                seq,
                ack_num,
                frame_type: FrameType::Ack,
            },
            payload: vec![],
        }
    }

    /// Create a heartbeat frame to keep middleboxes happy.
    pub fn new_heartbeat(seq: u64) -> Self {
        Self {
            header: FrameHeader {
                seq,
                ack_num: 0,
                frame_type: FrameType::Heartbeat,
            },
            payload: vec![],
        }
    }
}

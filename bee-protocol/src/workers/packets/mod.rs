// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO document

mod header;
mod heartbeat;
mod message;
mod message_request;
mod milestone_request;
mod tlv;

use std::ops::Range;

pub(crate) use self::{
    header::{HeaderPacket, HEADER_SIZE},
    heartbeat::HeartbeatPacket,
    message::MessagePacket,
    message_request::MessageRequestPacket,
    milestone_request::MilestoneRequestPacket,
    tlv::{tlv_from_bytes, tlv_to_bytes, Error as TlvError},
};

/// A trait describing the behavior of a packet.
///
/// This trait is protocol agnostic and only provides serialization and deserialization to and from byte buffers.
/// It should not be used as is but rather be paired with a higher layer - like a type-length-value encoding - and as
/// such does not provide any bounds check on inputs/outputs buffers.
pub(crate) trait Packet {
    /// The unique identifier of the packet within the protocol.
    const ID: u8;

    /// Returns the size range of the packet as it can be compressed.
    fn size_range() -> Range<usize>;

    /// Deserializes a byte buffer into a packet.
    ///
    /// # Arguments
    ///
    /// * `bytes`   -   The byte buffer to deserialize from.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer has an invalid size.
    /// The size of the buffer should be within the range returned by the `size_range` method.
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Returns the size of the packet.
    fn size(&self) -> usize;

    /// Serializes a packet to a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `bytes`   -   The byte buffer to serialize into.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer has an invalid size.
    /// The size of the buffer should be equal to the one returned by the `size` method.
    fn to_bytes(&self, bytes: &mut [u8]);
}

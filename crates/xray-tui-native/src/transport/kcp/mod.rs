//! mKCP transport: xray's fork of the KCP reliable-stream protocol over UDP
//! (SP4). Task 1 lands the wire codec; the session state machine is Task 2.

pub mod wire;

pub use wire::{Command, Segment, SegmentOption, encode_segment, parse_datagram};

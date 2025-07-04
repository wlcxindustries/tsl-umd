//! This crate contains an implementation of TSL UMD protocols.
//!
//! These are a family of commonly-used protocols for controlling tally (on air) lights,
//! particularly on multiviewers/under monitor displays/broadcast monitors. It was originally
//! used over serial, but is commonly also sent over IP. v5 explicitly supports this.
//!
//! ```rust
//!   use tsl_umd::v3_1::{TSL31Packet, PACKET_LENGTH_31};
//!   // Build a new packet in a buffer:
//!   let mut raw = [0u8; PACKET_LENGTH_31];
//!   let mut p = TSL31Packet::new_unchecked(&mut raw);
//!   p.set_address(13);
//!   p.set_display_data("hello");
//!   p.set_tally([true, false, false, false]);
//!
//!   // Take a buffer and check that it's a valid packet, then access fields within it:
//!   let packet = TSL31Packet::new_checked(raw).unwrap();
//!   assert_eq!(packet.address(), 13);
//!   assert_eq!(packet.display_data(), "hello");
//!   assert!(packet.tally()[0]);
//! ````
#![no_std]
pub mod v3_1;

#[cfg(feature = "std")]
extern crate std;

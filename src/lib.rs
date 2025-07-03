//! This crate contains an implementation of TSL UMD protocols.
//!
//! These are a family of commonly-used protocols for controlling tally (on air) lights,
//! particularly on multiviewers/under monitor displays/broadcast monitors. It was originally
//! used over serial, but is commonly also sent over IP. v5 explicitly supports this.
//!
//! An existing buffer of bytes is interpreted as a TSL UMD packet with the `new_checked` functions:
//! ```rust
//!   use tsl_umd::v3_1::{TSL31Packet, PACKET_LENGTH_31};
//!   let raw = [0u8; PACKET_LENGTH_31];
//!   let mut p = TSL31Packet::new_unchecked(raw);
//!   p.set_address(13);
//!   p.set_display_data("hello");
//!   p.set_tally([true, false, false, false]);
//!   assert_eq!(p.address(), 13);
//!   assert_eq!(p.display_data(), "hello");
//!   assert!(p.tally()[0]);
//! ````
#![no_std]
pub mod v3_1;

#[cfg(feature = "std")]
extern crate std;

//! This crate contains an implementation of TSL UMD protocols.
//!
//! These are a family of commonly-used protocols for controlling tally (on air) lights,
//! particularly on multiviewers/under monitor displays/broadcast monitors. It was originally
//! used over serial, but is commonly also sent over IP. v5 explicitly supports this.
//!
//! An existing buffer of bytes is interpreted as a TSL UMD packet with the `new_checked` functions:
//! ```rust
//!   use tsl_umd::v3_1::TSL31Packet;
//!   let raw = [0x8D, 0x19, b'h', b'e', b'l', b'l', b'o', 0,0,0,0,0,0,0,0,0,0,0];
//!   let p = TSL31Packet::new_checked(raw).unwrap();
//!   assert_eq!(p.address(), 13);
//!   assert_eq!(p.display_data(), "hello");
//!   assert!(p.tally()[0]);
//! ````
#![no_std]
pub mod v3_1;

#[cfg(feature = "std")]
extern crate std;

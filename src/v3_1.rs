//! Version 3.1 implementation
use core::{fmt::Display, ops::RangeInclusive};

/// TSL 3.1 packets are always 18 bytes long
pub const PACKET_LENGTH_31: usize = 18;

/// Range of values valid as display data (printable bytes)
pub const VALID_DISPLAY: RangeInclusive<u8> = 0x20..=0x7F;

/// A wrapper around a byte slice reference representing a TSL v3.1 Packet
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TSL31Packet<T: AsRef<[u8]>> {
    pub(crate) buf: T,
}

/// Tally light brightness, in 4 discrete steps
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Brightness {
    Zero,
    OneSeventh,
    OneHalf,
    Full,
}

impl Into<u8> for Brightness {
    /// The brightness value as a u8
    fn into(self) -> u8 {
        match self {
            Self::Zero => 0,
            Self::OneSeventh => 36, // Approx
            Self::OneHalf => 128,
            Self::Full => 255,
        }
    }
}

impl Display for Brightness {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Zero => "0",
                Self::OneSeventh => "1/7",
                Self::OneHalf => "1/2",
                Self::Full => "1",
            }
        )
    }
}

/// Packet checking error
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The first bit of the address isn't set - so it isn't a valid address
    AddressInvalid,
    /// The packet was an unexpected length
    BadLength { expected: usize, got: usize },
    /// Bad (non-ascii) bytes in the display data field.
    BadDisplayData { position: u8 },
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AddressInvalid => write!(f, "AddressInvalid"),
            Self::BadLength { expected, got } => {
                write!(f, "BadLength: expected {expected}, got {got}")
            }
            Self::BadDisplayData { position } => {
                write!(f, "BadDisplayData at position {position}")
            }
        }
    }
}

pub(crate) mod fields {
    use core::ops::Range;

    use super::PACKET_LENGTH_31;

    pub(crate) const ADDRESS: usize = 0;
    pub(crate) const CONTROL: usize = 1;
    pub(crate) const DISPLAY_DATA: Range<usize> = 2..PACKET_LENGTH_31;
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl<T> TSL31Packet<T>
where
    T: AsRef<[u8]>,
{
    /// Summon a packet from the given bytes without checking it.
    pub fn new_unchecked(buf: T) -> Self {
        Self { buf }
    }
    /// Validate the the given bytes are a packet and return it, or an error
    pub fn new_checked(buf: T) -> Result<Self, Error> {
        let p = Self::new_unchecked(buf);
        p.validate()?;
        Ok(p)
    }

    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.buf.as_ref().len() != PACKET_LENGTH_31 {
            return Err(Error::BadLength {
                expected: PACKET_LENGTH_31,
                got: self.buf.as_ref().len(),
            });
        }
        if self.buf.as_ref()[fields::ADDRESS] & 0x80 == 0 {
            return Err(Error::AddressInvalid);
        }
        for (i, b) in self.buf.as_ref()[fields::DISPLAY_DATA].iter().enumerate() {
            // N.B technically null bytes violates the spec, which clearly states that
            // only ascii in the range 0x20..=0x7f is valid. However at least one OSS
            // tally tool pads with null so... here we are
            if !((0x20..=0x7f).contains(b) || *b == 0) {
                // Safe to cast to u8 as len will never exceed 18
                return Err(Error::BadDisplayData { position: i as u8 });
            }
        }
        Ok(())
    }

    /// Consumes self, returning the inner bytes
    pub fn inner(self) -> T {
        self.buf
    }

    /// Return the display data as a string, with trailing space/null bytes removed
    pub fn display_data(&self) -> &str {
        // Use up to the first null byte, or the whole 16 chars
        let range = self.buf.as_ref()[fields::DISPLAY_DATA]
            .iter()
            .position(|c| *c == 0)
            .map(|e| fields::DISPLAY_DATA.start..e + fields::DISPLAY_DATA.start)
            .unwrap_or(fields::DISPLAY_DATA);
        // This is checked in `new_checked` so is safe to do
        unsafe { str::from_utf8_unchecked(&self.buf.as_ref()[range]).trim_end() }
    }

    /// The packet address, from `0x00..=0x7E`
    pub fn address(&self) -> u8 {
        self.buf.as_ref()[fields::ADDRESS] & 0x7f
    }

    /// Tally states, 4 channels
    pub fn tally(&self) -> [bool; 4] {
        let ctrl = self.buf.as_ref()[fields::CONTROL];
        [
            ctrl & 0b1 != 0,
            ctrl & 0b10 != 0,
            ctrl & 0b100 != 0,
            ctrl & 0b1000 != 0,
        ]
    }

    /// Tally brightness
    pub fn brightness(&self) -> Brightness {
        match (self.buf.as_ref()[fields::CONTROL] >> 4) & 0x3 {
            0 => Brightness::Zero,
            0b01 => Brightness::OneSeventh,
            0b10 => Brightness::OneHalf,
            0b11 => Brightness::Full,
            _ => unreachable!(),
        }
    }
}

impl<T> TSL31Packet<T>
where
    T: AsMut<[u8]> + AsRef<[u8]>,
{
    /// Set the address. Return Err(()) if the addr is out of range
    pub fn set_address(&mut self, addr: u8) -> Result<(), ()> {
        if !(0x0..=0x7E).contains(&addr) {
            return Err(());
        }
        self.buf.as_mut()[fields::ADDRESS] = addr + 0x80;
        Ok(())
    }

    /// Set the tally state
    pub fn set_tally(&mut self, state: [bool; 4]) {
        let b: u8 = state
            .iter()
            .enumerate()
            .map(|(i, v)| if *v { 1 << i } else { 0 })
            .sum();
        self.buf.as_mut()[fields::CONTROL] = (self.buf.as_ref()[fields::CONTROL] & 0xf0) | b;
    }

    pub fn set_brightness(&mut self, brightness: Brightness) {
        let b = match brightness {
            Brightness::Zero => 0,
            Brightness::OneSeventh => 0b01 << 4,
            Brightness::OneHalf => 0b10 << 4,
            Brightness::Full => 0b11 << 4,
        };
        self.buf.as_mut()[fields::CONTROL] = (self.buf.as_ref()[fields::CONTROL] & 0x0f) | b;
    }

    /// Set the display data. Panics if length > 16 or string does not contain printable ascii
    pub fn set_display_data<'a, S>(&mut self, s: S)
    where
        S: Into<&'a str>,
    {
        // TODO: don't panic
        let s: &str = s.into();
        if s.len() > 16 {
            panic!("String must not be longer than 16 chars");
        }
        if !s.as_bytes().iter().all(|c| VALID_DISPLAY.contains(c)) {
            panic!("String must be printable ascii only");
        }
        // Length is checked above, so safe to do this
        self.buf.as_mut()[fields::DISPLAY_DATA.start..fields::DISPLAY_DATA.start + s.len()]
            .copy_from_slice(s.as_bytes());
    }
}

impl<T> Display for TSL31Packet<T>
where
    T: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "addr={}, 1={}, 2={}, 3={}, 4={}, brightness={}, display={}",
            self.address(),
            self.tally()[0],
            self.tally()[1],
            self.tally()[2],
            self.tally()[3],
            self.brightness(),
            self.display_data()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const VALID_RAW: [u8; PACKET_LENGTH_31] = [
        0x80 + 0x69,
        0b00011001,
        b'h',
        b'e',
        b'l',
        b'l',
        b'o',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
        b' ',
    ];

    #[test]
    fn test_parse() {
        let p = TSL31Packet::new_checked(VALID_RAW).unwrap();
        assert_eq!(p.address(), 0x69);
        assert_eq!(p.tally(), [true, false, false, true]);
        assert_eq!(p.brightness(), Brightness::OneSeventh);
        assert_eq!(p.display_data(), "hello");
    }

    #[test]
    fn error_bad_length() {
        assert_eq!(
            TSL31Packet::new_checked(&[]),
            Err(Error::BadLength {
                expected: PACKET_LENGTH_31,
                got: 0
            })
        );
        assert_eq!(
            TSL31Packet::new_checked(&[0; PACKET_LENGTH_31 + 1]),
            Err(Error::BadLength {
                expected: PACKET_LENGTH_31,
                got: 19
            })
        );
    }

    #[test]
    fn error_bad_address() {
        let mut bad_raw = VALID_RAW;
        bad_raw[0] = 0x13;
        assert_eq!(
            TSL31Packet::new_checked(bad_raw),
            Err(Error::AddressInvalid)
        );
    }

    #[test]
    fn error_bad_display() {
        let mut bad_raw = VALID_RAW;
        let ohno = "oh no 🤔".as_bytes();
        bad_raw[2..2 + ohno.len()].copy_from_slice(ohno);
        assert_eq!(
            TSL31Packet::new_checked(bad_raw),
            Err(Error::BadDisplayData { position: 6 })
        );
    }

    #[test]
    fn test_set_address() {
        let buf = [0u8; PACKET_LENGTH_31];
        let mut p = TSL31Packet::new_unchecked(buf);
        p.set_address(42).unwrap();
        assert_eq!(p.address(), 42);
        assert!(p.set_address(234).is_err());
    }

    #[test]
    fn test_set_tally() {
        let buf = [0u8; PACKET_LENGTH_31];
        let mut p = TSL31Packet::new_unchecked(buf);
        for perm in [
            [false, false, false, false],
            [true, false, false, false],
            [false, true, false, false],
            [false, false, true, false],
            [false, false, false, true],
        ] {
            p.set_tally(perm);
            assert_eq!(p.tally(), perm);
        }
    }

    #[test]
    fn test_set_brightness() {
        let buf = [0u8; PACKET_LENGTH_31];
        let mut p = TSL31Packet::new_unchecked(buf);
        for b in [
            Brightness::Zero,
            Brightness::OneSeventh,
            Brightness::OneHalf,
            Brightness::Full,
        ] {
            p.set_brightness(b);
            assert_eq!(p.brightness(), b);
        }
    }

    #[test]
    fn test_set_display_data() {
        let buf = [0u8; PACKET_LENGTH_31];
        let mut p = TSL31Packet::new_unchecked(buf);
        for s in ["", "hello there", "1234567890=+!)()"] {
            p.set_display_data(s);
            assert_eq!(p.display_data(), s);
        }
    }
}

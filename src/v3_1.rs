//! Version 3.1 implementation
use core::fmt::Display;

/// TSL 3.1 packets are always 18 bytes long
pub const PACKET_LENGTH_31: usize = 18;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TSL31Packet<T: AsRef<[u8]>> {
    pub(crate) buf: T,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Brightness {
    Zero,
    OneSeventh,
    OneHalf,
    Full,
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

/// Protocol parsing error
#[derive(Debug, PartialEq, Eq)]
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

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl<T> TSL31Packet<T>
where
    T: AsRef<[u8]>,
{
    pub fn new_checked(buf: T) -> Result<Self, Error> {
        Self::validate(&buf)?;
        Ok(Self { buf })
    }

    pub(crate) fn validate(buf: &T) -> Result<(), Error> {
        if buf.as_ref().len() != PACKET_LENGTH_31 {
            return Err(Error::BadLength {
                expected: PACKET_LENGTH_31,
                got: buf.as_ref().len(),
            });
        }
        if buf.as_ref()[0] & 0x80 == 0 {
            return Err(Error::AddressInvalid);
        }
        for (i, b) in buf.as_ref()[2..].iter().enumerate() {
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

    /// Consumes self, returning the inner buffer
    pub fn inner(self) -> T {
        self.buf
    }

    /// Return the display data as a string, with trailing space/null bytes removed
    pub fn display_data(&self) -> &str {
        // Use up to the first null byte, or the whole 16 chars
        let range = self
            .buf
            .as_ref()
            .iter()
            .position(|c| *c == 0)
            .map(|e| 2..e)
            .unwrap_or(2..self.buf.as_ref().len());
        // This is checked in `new_checked` so is safe to do
        unsafe { str::from_utf8_unchecked(&self.buf.as_ref()[range]).trim_end() }
    }

    /// The packet address, from `0x00..=0x7E`
    pub fn address(&self) -> u8 {
        self.buf.as_ref()[0] & 0x7f
    }

    /// Tally states, 4 channels
    pub fn tally(&self) -> [bool; 4] {
        let ctrl = self.buf.as_ref()[1];
        [
            ctrl & 0b1 != 0,
            ctrl & 0b10 != 0,
            ctrl & 0b100 != 0,
            ctrl & 0b1000 != 0,
        ]
    }

    /// Tally brightness
    pub fn brightness(&self) -> Brightness {
        match (self.buf.as_ref()[1] >> 4) & 0b11 {
            0 => Brightness::Zero,
            0b10 => Brightness::OneHalf,
            0b01 => Brightness::OneSeventh,
            0b11 => Brightness::Full,
            _ => unreachable!(),
        }
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
}

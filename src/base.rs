use anyhow::{bail, Result};

#[derive(PartialEq, Debug)]
pub enum Endianness {
    Big,
    Small,
}

impl Endianness {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(match bytes {
            [73, 73] => Self::Small,
            [77, 77] => Self::Big,
            _ => bail!("Unrecognised endian bytes: {:?}", bytes)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::base::Endianness;

    #[test]
    fn test() {
        assert!(Endianness::from_bytes(&[77, 77]).is_ok_and(|e| e == Endianness::Big));
        assert!(Endianness::from_bytes(&[73, 73]).is_ok_and(|e| e == Endianness::Small));
        assert!(Endianness::from_bytes(&[0, 73]).is_err());
        assert!(Endianness::from_bytes(&[77, 77, 0]).is_err());
    }
}
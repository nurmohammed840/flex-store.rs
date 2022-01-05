use crate::endian::Endian;
use std::io::{Cursor, Read, Result, Write};

pub trait Reader {
    fn next(&mut self) -> Option<u8>;
    fn buf<const S: usize>(&mut self) -> Result<[u8; S]>;
    fn get<E>(&mut self) -> Result<E>
    where
        E: Endian,
        [u8; E::NBYTES]:;
}
pub trait Writer: Reader {
    fn set<E>(&mut self, value: E) -> Result<()>
    where
        E: Endian,
        [u8; E::NBYTES]:;
}

impl<T: AsRef<[u8]>> Reader for Cursor<T> {
    #[inline]
    fn next(&mut self) -> Option<u8> {
        let pos = self.position();
        let byte = *self.get_ref().as_ref().get(pos as usize)?;
        self.set_position(pos + 1);
        Some(byte)
    }
    #[inline]
    fn buf<const S: usize>(&mut self) -> Result<[u8; S]> {
        let mut buf = [0u8; S];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }
    #[inline]
    fn get<R>(&mut self) -> Result<R>
    where
        R: Endian,
        [u8; R::NBYTES]:,
    {
        let bytes = self.buf()?;
        Ok(R::from_bytes_le(bytes))
    }
}

impl Writer for Cursor<&mut [u8]> {
    fn set<R>(&mut self, value: R) -> Result<()>
    where
        R: Endian,
        [u8; R::NBYTES]:,
    {
        self.write_all(&R::to_bytes_le(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let mut reader = Cursor::new([1, 2]);
        assert_eq!(Some(1), reader.next());
        assert_eq!(Some(2), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn test_reader_buf() {
        let mut reader = Cursor::new([1, 2, 3, 4]);
        assert_eq!([1, 2, 3, 4], reader.buf::<4>().unwrap());
        assert_eq!(true, reader.buf::<5>().is_err());
    }

    #[test]
    fn test_writer() {
        let mut buf = [0u8; 7];
        let mut writer = Cursor::new(&mut buf[..]);
        writer.set(1_u8).unwrap();
        writer.set::<u16>(2).unwrap();
        writer.set::<u32>(3).unwrap();
        assert_eq!(true, writer.set::<u128>(0).is_err());

        writer.set_position(0);
        assert_eq!(1, writer.get::<u8>().unwrap());
        assert_eq!(2, writer.get::<u16>().unwrap());
        assert_eq!(3, writer.get::<u32>().unwrap());
        assert_eq!(true, writer.get::<u128>().is_err());
    }
}

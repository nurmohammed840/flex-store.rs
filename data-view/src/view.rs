use crate::{DataView, Endian};

/// This struct represents a data view for reading and writing data in a byte array.
/// When read/write, This increment current offset by the size of the value.
///
/// # Examples
///
/// ```
/// use data_view::View;
///
/// let mut view = View::new([0; 4]);
///
/// view.set::<u16>(42);
/// view.offset = 0;
/// assert_eq!(view.get::<u16>(), 42);
/// ```
pub struct View<T> {
    pub data:   T,
    pub offset: usize,
}

impl<T: AsRef<[u8]>> View<T> {
    /// Creates a new `View` from a byte array.
    #[inline(always)]
    pub fn new(data: T) -> Self { Self { data, offset: 0 } }

    /// Returns remaining slice from the current offset.
    #[inline(always)]
    pub fn remaining_slice(&self) -> &[u8] {
        let data = self.data.as_ref();
        &data[self.offset.min(data.len())..]
    }

    /// Get next byte from the current offset.
    #[inline(always)]
    pub fn next(&mut self) -> Option<u8> {
        let byte = *self.data.as_ref().get(self.offset)?;
        self.offset += 1;
        Some(byte)
    }

    /// Reads a value of type `E` from the data view. where `E` implements `Endian`.
    /// And updates the current offset: offset + size of the value.
    ///
    /// # Panics
    /// Panics if the offset is out of bounds.
    #[inline(always)]
    pub fn get<E>(&mut self) -> E
    where
        E: Endian,
        [u8; E::NBYTES]:,
    {
        let value = self.data.as_ref().read(self.offset);
        self.offset += E::NBYTES;
        value
    }

    /// Create a buffer and returns it, from the current offset.
    /// And increments the current offset: offset + size of the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_view::View;
    ///
    /// let mut view = View::new([0; 16]);
    ///
    /// let buf = view.get_buf::<2>();
    /// assert_eq!(buf.len(), 2);
    /// ```
    ///
    /// # Panics
    /// Panics if the offset is out of bounds.
    #[inline(always)]
    pub fn get_buf<const N: usize>(&mut self) -> [u8; N] {
        let mut buf = [0; N];
        buf.copy_from_slice(&self.data.as_ref()[self.offset..self.offset + N]);
        self.offset += N;
        buf
    }
}

impl<T: AsMut<[u8]>> View<T> {
    /// Writes a value of type `E` to the data view. where `E` is a type that implements `Endian`.
    /// And updates the current offset: offset + size of the value.
    ///
    /// # Panics
    /// Panics if the offset is out of bounds.
    #[inline(always)]
    pub fn set<E>(&mut self, value: E)
    where
        E: Endian,
        [u8; E::NBYTES]:,
    {
        self.data.as_mut().write(self.offset, value);
        self.offset += E::NBYTES;
    }
}

use crate::endian::Endian;

/// A data view for reading and writing data in a byte array.
///
/// # Examples
///
/// ```
/// use data_view::DataView;
/// 
/// let mut buf = [0; 16];
/// 
/// buf.write::<u16>(1, 42);
/// assert_eq!(buf.read::<u16>(1), 42);
/// ```
/// 
/// # Panics
/// Panics if the offset is out of bounds.
pub trait DataView {
    /// Reads a value of type `E` from the data view. where `E` implements `Endian`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use data_view::DataView;
    /// 
    /// let mut buf: [u8; 2] = [42, 0];
    /// 
    /// assert_eq!(buf.read::<u8>(0), 42);
    /// assert_eq!(buf.read::<u8>(1), 0);
    /// ```
    /// 
    /// # Panics
    /// Panics if the offset is out of bounds.
    fn read<E>(&self, offset: usize) -> E
    where
        E: Endian,
        [u8; E::NBYTES]:;

    /// Writes a value of type `E` to the data view. where `E` is a type that implements `Endian`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use data_view::DataView;
    /// 
    /// let mut buf: [u8; 2] = [0; 2];
    /// 
    /// buf.write::<u8>(0, 42);
    /// assert_eq!(buf, [42, 0]);
    /// ```
    /// 
    /// # Panics
    /// Panics if the offset is out of bounds.
    fn write<E>(&mut self, offset: usize, value: E)
    where
        E: Endian,
        [u8; E::NBYTES]:;
}

impl DataView for [u8] {
    #[inline(always)]
    fn read<T>(&self, offset: usize) -> T
    where
        T: Endian,
        [u8; T::NBYTES]:,
    {
        #[cfg(not(any(feature = "BE", feature = "NE")))]
        return T::from_bytes_le(self[offset..offset + T::NBYTES].try_into().unwrap());
        #[cfg(feature = "BE")]
        return T::from_bytes_be(self[offset..offset + T::NBYTES].try_into().unwrap());
        #[cfg(feature = "NE")]
        return T::from_bytes_ne(self[offset..offset + T::NBYTES].try_into().unwrap());
    }

    #[inline(always)]
    fn write<T>(&mut self, offset: usize, value: T)
    where
        T: Endian,
        [u8; T::NBYTES]:,
    {
        #[cfg(not(any(feature = "BE", feature = "NE")))]
        self[offset..offset + T::NBYTES].copy_from_slice(&value.to_bytes_le());
        #[cfg(feature = "BE")]
        self[offset..offset + T::NBYTES].copy_from_slice(&value.to_bytes_be());
        #[cfg(feature = "NE")]
        self[offset..offset + T::NBYTES].copy_from_slice(&value.to_bytes_ne());
    }
}

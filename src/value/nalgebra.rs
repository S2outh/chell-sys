use super::ChellValue;
use super::ChellValueError;

use nalgebra as na;

macro_rules! newtype_value {
    ($field: tt, $inner: ty, $constructor: path, $($interface: tt)*) => {
        $($interface)* {
            const MAX_BYTE_SIZE: usize = <$inner>::MAX_BYTE_SIZE;

            #[inline]
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
            where
                Self: Sized,
            {
                let (len, value) = <$inner>::read(bytes)?;
                Ok((len, $constructor(value)))
            }
            #[inline]
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                self.$field.write(mem)
            }
        }
    };
}

newtype_value!(0, [[T; R]; C], Self,
    impl<T, const R: usize, const C: usize> ChellValue for na::ArrayStorage<T, R, C>
        where T: ChellValue,
);

newtype_value!(data, S, Self::from_data,
    impl<T, R, C, S> ChellValue for na::Matrix<T, R, C, S>
        where
            S: ChellValue,
            R: na::Dim,
            C: na::Dim,
            S: na::RawStorage<T, R, C>
);

newtype_value!(coords, na::Vector4<T>, Self::from_vector,
    impl<T> ChellValue for na::Quaternion<T>
        where T: ChellValue,
);

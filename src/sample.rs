use num_traits::{Float, PrimInt};

/// 24 bit signed integer, little endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
#[derive(Debug)]
pub struct I24LE<U>(U);

/// 24 bit signed integer, big endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
#[derive(Debug)]
pub struct I24BE<U>(U);

/// 24 bit unsigned integer, little endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
pub struct U24LE<U>(U);

/// 24 bit unsigned integer, big endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
pub struct U24BE<U>(U);

/// 32 bit signed integer, little endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct I32LE([u8; 4]);

/// 32 bit signed integer, big endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct I32BE([u8; 4]);

/// 64 bit signed integer, little endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct I64LE([u8; 8]);

/// 64 bit signed integer, big endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct I64BE([u8; 8]);

/// 16 bit signed integer, little endian. Stored as 2 bytes.
#[derive(Debug)]
pub struct I16LE([u8; 2]);

/// 16 bit signed integer, big endian. Stored as 2 bytes.
#[derive(Debug)]
pub struct I16BE([u8; 2]);

/// 32 bit unsigned integer, little endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct U32LE([u8; 4]);

/// 32 bit unsigned integer, big endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct U32BE([u8; 4]);

/// 64 bit unsigned integer, little endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct U64LE([u8; 8]);

/// 64 bit unsigned integer, big endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct U64BE([u8; 8]);

/// 16 bit unsigned integer, little endian. Stored as 2 bytes.
#[derive(Debug)]
pub struct U16LE([u8; 2]);

/// 16 bit unsigned integer, big endian. Stored as 2 bytes.
#[derive(Debug)]
pub struct U16BE([u8; 2]);

/// 32 bit floating point, little endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct F32LE([u8; 4]);

/// 32 bit floating point, big endian. Stored as 4 bytes.
#[derive(Debug)]
pub struct F32BE([u8; 4]);

/// 64 bit floating point, little endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct F64LE([u8; 8]);

/// 64 bit floating point, big endian. Stored as 8 bytes.
#[derive(Debug)]
pub struct F64BE([u8; 8]);

/// Convert a float to an integer, clamp at the min and max limits of the integer.
fn to_clamped_int<T: Float, U: PrimInt>(value: T, converted: Option<U>) -> ConversionResult<U> {
    if let Some(val) = converted {
        return ConversionResult {
            clipped: false,
            value: val,
        };
    }
    if value.is_nan() {
        return ConversionResult {
            clipped: true,
            value: U::zero(),
        };
    }
    if value > T::zero() {
        return ConversionResult {
            clipped: true,
            value: U::max_value(),
        };
    }
    ConversionResult {
        clipped: true,
        value: U::min_value(),
    }
}

/// A conversion result, containing the resulting value as `value`
/// and a boolean `clipped` indicating if the value was clipped during conversion.
pub struct ConversionResult<T> {
    pub clipped: bool,
    pub value: T,
}

/// A trait for converting a given sample type to and from floating point values.
/// The floating point values use the range -1.0 to +1.0.
/// When converting to/from signed integers, the range does not include +1.0.
/// For example, an 8-bit signed integer supports the range -128 to +127.
/// When these values are converted to float, 0 becomes 0.0,
/// -128 becomes -1.0, and 127 becomes 127/128 ≈ 0.992.
/// Unsigned integers are also converted to the same -1.0 to +1.0 range.
/// For an 8-but unsigned integer, 128 is the center point and becomes 0.0.
/// The value 0 becomes -1.0, and 255 becomes 127/128 ≈ 0.992.
pub trait RawSample
where
    Self: Sized,
{
    /// Convert the sample value to a float in the range -1.0 .. +1.0.
    fn to_scaled_float<T: Float>(&self) -> T;

    /// Convert a float in the range -1.0 .. +1.0 to a sample value.
    /// Values outside the allowed range are clipped to the nearest limit.
    fn from_scaled_float<T: Float>(value: T) -> ConversionResult<Self>;
}

/// A trait for converting samples stored as raw bytes into a numerical type.
/// Each implementation defines the associated type `NumericType`,
/// which is the nearest matching numeric type for the original format.
/// If a direct match exists, this is used.
/// For example signed 16 bit integer samples use [i16].
/// For formats that don't have a direct match,
/// the next larger numeric type is used.
/// For example for 24 bit signed integers,
/// this means [i32].
/// The values are scaled to use the full range of the `NumericType`
/// associated type.
pub trait BytesSample {
    /// The closest matching numeric type.
    type NumericType;

    /// The number of bytes making up each sample value.
    const BYTES_PER_SAMPLE: usize;

    /// Create a new ByteSample from a slice of raw bytes.
    /// The slice length must be at least the number of bytes
    /// for a sample value.
    fn from_slice(bytes: &[u8]) -> Self;

    /// Return the raw bytes as a slice.
    fn as_slice(&self) -> &[u8];

    /// Convert the raw bytes to a numerical value.
    fn to_number(&self) -> Self::NumericType;

    /// Convert a numerical value to raw bytes.
    fn from_number(value: Self::NumericType) -> Self;
}

macro_rules! rawsample_for_int {
    ($type:ident, $to:ident) => {
        impl RawSample for $type {
            fn to_scaled_float<T: Float>(&self) -> T {
                T::from(*self).unwrap() / (T::from($type::MAX).unwrap() + T::one())
            }

            fn from_scaled_float<T: Float>(value: T) -> ConversionResult<Self> {
                let scaled = value * (T::from($type::MAX).unwrap() + T::one());
                let converted = scaled.$to();
                to_clamped_int(scaled, converted)
            }
        }
    };
}

rawsample_for_int!(i8, to_i8);
rawsample_for_int!(i16, to_i16);
rawsample_for_int!(i32, to_i32);
rawsample_for_int!(i64, to_i64);

macro_rules! rawsample_for_uint {
    ($type:ident, $to:ident) => {
        impl RawSample for $type {
            fn to_scaled_float<T: Float>(&self) -> T {
                let max_ampl = (T::from($type::MAX).unwrap() + T::one()) / T::from(2).unwrap();
                (T::from(*self).unwrap() - max_ampl) / max_ampl
            }

            fn from_scaled_float<T: Float>(value: T) -> ConversionResult<Self> {
                let max_ampl = (T::from($type::MAX).unwrap() + T::one()) / T::from(2).unwrap();
                let scaled = value * max_ampl + max_ampl;
                let converted = scaled.$to();
                to_clamped_int(scaled, converted)
            }
        }
    };
}

rawsample_for_uint!(u8, to_u8);
rawsample_for_uint!(u16, to_u16);
rawsample_for_uint!(u32, to_u32);
rawsample_for_uint!(u64, to_u64);

macro_rules! rawsample_for_float {
    ($type:ident, $to:ident) => {
        impl RawSample for $type {
            fn to_scaled_float<T: Float>(&self) -> T {
                T::from(*self).unwrap_or(T::zero())
            }

            fn from_scaled_float<T: Float>(value: T) -> ConversionResult<Self> {
                // TODO clip here
                ConversionResult {
                    clipped: false,
                    value: value.$to().unwrap_or(0.0),
                }
            }
        }
    };
}

rawsample_for_float!(f32, to_f32);
rawsample_for_float!(f64, to_f64);

// 24 bit formats, needs more work than others
// because they don't map directly to a normal numerical type,

/// 24 bit signed integer, little endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for I24LE<[u8; 4]> {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = 4;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[0], self.0[1], self.0[2]];
        i32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([bytes[1], bytes[2], bytes[3], 0])
    }
}

/// 24 bit signed integer, little endian, stored as 3 bytes without padding.
impl BytesSample for I24LE<[u8; 3]> {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = 3;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[0], self.0[1], self.0[2]];
        i32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([bytes[1], bytes[2], bytes[3]])
    }
}

/// 24 bit signed integer, big endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for I24BE<[u8; 4]> {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = 4;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[1], self.0[2], self.0[3], 0];
        i32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([0, bytes[0], bytes[1], bytes[2]])
    }
}

/// 24 bit signed integer, big endian, stored as 3 bytes without padding.
impl BytesSample for I24BE<[u8; 3]> {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = 3;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[0], self.0[1], self.0[2], 0];
        i32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([bytes[0], bytes[1], bytes[2]])
    }
}

/// 24 bit unsigned integer, little endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for U24LE<[u8; 4]> {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = 4;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[0], self.0[1], self.0[2]];
        u32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([bytes[1], bytes[2], bytes[3], 0])
    }
}

/// 24 bit unsigned integer, little endian, stored as 3 bytes without padding.
impl BytesSample for U24LE<[u8; 3]> {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = 3;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[0], self.0[1], self.0[2]];
        u32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([bytes[1], bytes[2], bytes[3]])
    }
}

/// 24 bit unsigned integer, big endian, stored as 4 bytes. The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for U24BE<[u8; 4]> {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = 4;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[1], self.0[2], self.0[3], 0];
        u32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([0, bytes[0], bytes[1], bytes[2]])
    }
}

/// 24 bit unsigned integer, big endian, stored as 3 bytes without padding.
impl BytesSample for U24BE<[u8; 3]> {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = 3;

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[0], self.0[1], self.0[2], 0];
        u32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([bytes[0], bytes[1], bytes[2]])
    }
}

macro_rules! bytessample_for_newtype {
    ($type:ident, $newtype:ident, $from:ident, $to:ident) => {
        impl BytesSample for $newtype {
            type NumericType = $type;
            const BYTES_PER_SAMPLE: usize = core::mem::size_of::<$type>();

            fn from_slice(bytes: &[u8]) -> Self {
                Self(bytes.try_into().unwrap())
            }

            fn as_slice(&self) -> &[u8] {
                &self.0
            }

            fn to_number(&self) -> Self::NumericType {
                $type::$from(self.0)
            }

            fn from_number(value: Self::NumericType) -> Self {
                Self(value.$to())
            }
        }
    };
}

bytessample_for_newtype!(i64, I64LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u64, U64LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i64, I64BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u64, U64BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(i16, I16LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u16, U16LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i16, I16BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u16, U16BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(i32, I32LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u32, U32LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i32, I32BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u32, U32BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(f32, F32LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(f32, F32BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(f64, F64LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(f64, F64BE, from_be_bytes, to_be_bytes);

impl<V> RawSample for V
where
    V: BytesSample,
    <V as BytesSample>::NumericType: RawSample,
{
    fn to_scaled_float<T: Float>(&self) -> T {
        let value = self.to_number();
        value.to_scaled_float()
    }

    fn from_scaled_float<T: Float>(value: T) -> ConversionResult<Self> {
        let value = <V as BytesSample>::NumericType::from_scaled_float(value);
        ConversionResult {
            clipped: value.clipped,
            value: V::from_number(value.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste::paste;

    macro_rules! assert_conversion_eq {
        ($result:expr, $value:expr, $clipped:expr, $desc:expr) => {
            assert_eq!($result.value, $value, $desc);
            assert_eq!($result.clipped, $clipped, $desc);
        };
    }

    macro_rules! test_to_signed_int {
        ($float:ty, $int:ident, $bits:expr) => {
            paste! {
                #[test]
                fn [< test_ $float _to_ $int >]() {
                    let val: $float = 0.25;
                    assert_conversion_eq!($int::from_scaled_float(val), 1 << ($bits - 3), false, "check +0.25");
                    let val: $float = -0.25;
                    assert_conversion_eq!($int::from_scaled_float(val), -1 << ($bits - 3), false, "check -0.25");
                    let val: $float = 1.1;
                    assert_conversion_eq!($int::from_scaled_float(val), $int::MAX, true, "clipped positive");
                    let val: $float = -1.1;
                    assert_conversion_eq!($int::from_scaled_float(val), $int::MIN, true, "clipped negative");
                }
            }
        };
    }

    macro_rules! test_to_unsigned_int {
        ($float:ty, $int:ident, $bits:expr) => {
            paste! {
                #[test]
                fn [< test_ $float _to_ $int >]() {
                    let val: $float = -0.5;
                    assert_conversion_eq!($int::from_scaled_float(val), 1 << ($bits - 2), false, "check -0.5");
                    let val: $float = 0.5;
                    assert_conversion_eq!($int::from_scaled_float(val), $int::MAX - (1 << ($bits - 2)) + 1, false, "check 0.5");
                    let val: $float = 1.1;
                    assert_conversion_eq!($int::from_scaled_float(val), $int::MAX, true, "clipped positive");
                    let val: $float = -1.1;
                    assert_conversion_eq!($int::from_scaled_float(val), $int::MIN, true, "clipped negative");
                }
            }
        };
    }

    test_to_signed_int!(f32, i8, 8);
    test_to_signed_int!(f64, i8, 8);
    test_to_signed_int!(f32, i16, 16);
    test_to_signed_int!(f64, i16, 16);
    test_to_signed_int!(f32, i32, 32);
    test_to_signed_int!(f64, i32, 32);
    test_to_signed_int!(f32, i64, 64);
    test_to_signed_int!(f64, i64, 64);

    test_to_unsigned_int!(f32, u8, 8);
    test_to_unsigned_int!(f64, u8, 8);
    test_to_unsigned_int!(f32, u16, 16);
    test_to_unsigned_int!(f64, u16, 16);
    test_to_unsigned_int!(f32, u32, 32);
    test_to_unsigned_int!(f64, u32, 32);
    test_to_unsigned_int!(f32, u64, 64);
    test_to_unsigned_int!(f64, u64, 64);

    macro_rules! test_from_signed_int {
        ($float:ty, $int:ident, $bits:expr) => {
            paste! {
                #[test]
                fn [< test_ $float _from_ $int >]() {
                    let val: $int = -1 << ($bits - 2);
                    assert_eq!(val.to_scaled_float::<$float>(), -0.5, "check -0.5");
                    let val: $int = 1 << ($bits - 2);
                    assert_eq!(val.to_scaled_float::<$float>(), 0.5, "check 0.5");
                    let val: $int = $int::MIN;
                    assert_eq!(val.to_scaled_float::<$float>(), -1.0, "negative limit");
                }
            }
        };
    }

    macro_rules! test_from_unsigned_int {
        ($float:ty, $int:ident, $bits:expr) => {
            paste! {
                #[test]
                fn [< test_ $float _from_ $int >]() {
                    let val: $int = 1 << ($bits - 2);
                    assert_eq!(val.to_scaled_float::<$float>(), -0.5, "check -0.5");
                    let val: $int = $int::MAX - (1 << ($bits - 2)) + 1;
                    assert_eq!(val.to_scaled_float::<$float>(), 0.5, "check 0.5");
                    let val: $int = 0;
                    assert_eq!(val.to_scaled_float::<$float>(), -1.0, "negative limit");
                }
            }
        };
    }

    test_from_signed_int!(f32, i8, 8);
    test_from_signed_int!(f64, i8, 8);
    test_from_signed_int!(f32, i16, 16);
    test_from_signed_int!(f64, i16, 16);
    test_from_signed_int!(f32, i32, 32);
    test_from_signed_int!(f64, i32, 32);
    test_from_signed_int!(f32, i64, 64);
    test_from_signed_int!(f64, i64, 64);

    test_from_unsigned_int!(f32, u8, 8);
    test_from_unsigned_int!(f64, u8, 8);
    test_from_unsigned_int!(f32, u16, 16);
    test_from_unsigned_int!(f64, u16, 16);
    test_from_unsigned_int!(f32, u32, 32);
    test_from_unsigned_int!(f64, u32, 32);
    test_from_unsigned_int!(f32, u64, 64);
    test_from_unsigned_int!(f64, u64, 64);

    #[test]
    fn test_to_clamped_int() {
        let converted = to_clamped_int::<f32, i32>(12345.0, Some(12345));
        assert_conversion_eq!(converted, 12345, false, "in range f32 i32");

        let converted = to_clamped_int::<f32, i32>(1.0e10, None);
        assert_conversion_eq!(converted, i32::MAX, true, "above range f32 i32");

        let converted = to_clamped_int::<f32, i32>(-1.0e10, None);
        assert_conversion_eq!(converted, i32::MIN, true, "below range f32 i32");

        let converted = to_clamped_int::<f64, i32>(12345.0, Some(12345));
        assert_conversion_eq!(converted, 12345, false, "in range f64 i32");

        let converted = to_clamped_int::<f64, i32>(1.0e10, None);
        assert_conversion_eq!(converted, i32::MAX, true, "above range f64 i32");

        let converted = to_clamped_int::<f64, i32>(-1.0e10, None);
        assert_conversion_eq!(converted, i32::MIN, true, "below range f64 i32");
    }

    #[test]
    fn test_to_clamped_uint() {
        let converted = to_clamped_int::<f32, u32>(12345.0, Some(12345));
        assert_conversion_eq!(converted, 12345, false, "in range f32 u32");

        let converted = to_clamped_int::<f32, u32>(1.0e10, None);
        assert_conversion_eq!(converted, u32::MAX, true, "above range f32 u32");

        let converted = to_clamped_int::<f32, u32>(-1.0, None);
        assert_conversion_eq!(converted, u32::MIN, true, "below range f32 u32");

        let converted = to_clamped_int::<f64, u32>(12345.0, Some(12345));
        assert_conversion_eq!(converted, 12345, false, "in range f64 u32");

        let converted = to_clamped_int::<f64, u32>(1.0e10, None);
        assert_conversion_eq!(converted, u32::MAX, true, "above range f64 u32");

        let converted = to_clamped_int::<f64, u32>(-1.0, None);
        assert_conversion_eq!(converted, u32::MIN, true, "below range f64 u32");
    }

    macro_rules! test_simple_int_bytes {
        ($number:ty, $wrapper:ident, $endian:ident) => {
            paste! {
                #[test]
                #[allow(non_snake_case)]
                fn [< test_ $wrapper >]() {
                    let number: $number = $number::MAX/5 * 4;
                    let wrapped = $wrapper(number.[< to_ $endian _bytes>]());
                    assert_eq!(number, wrapped.to_number());
                }
            }
        };
    }

    macro_rules! test_float_bytes {
        ($number:ty, $wrapper:ident, $endian:ident) => {
            paste! {
                #[test]
                #[allow(non_snake_case)]
                fn [< test_ $wrapper >]() {
                    let number: $number = 12345.0;
                    let wrapped = $wrapper(number.[< to_ $endian _bytes>]());
                    assert_eq!(number, wrapped.to_number());
                }
            }
        };
    }

    test_simple_int_bytes!(i16, I16LE, le);
    test_simple_int_bytes!(i16, I16BE, be);
    test_simple_int_bytes!(i32, I32LE, le);
    test_simple_int_bytes!(i32, I32BE, be);
    test_simple_int_bytes!(i64, I64LE, le);
    test_simple_int_bytes!(i64, I64BE, be);

    test_simple_int_bytes!(u16, U16LE, le);
    test_simple_int_bytes!(u16, U16BE, be);
    test_simple_int_bytes!(u32, U32LE, le);
    test_simple_int_bytes!(u32, U32BE, be);
    test_simple_int_bytes!(u64, U64LE, le);
    test_simple_int_bytes!(u64, U64BE, be);

    test_float_bytes!(f32, F32LE, le);
    test_float_bytes!(f32, F32BE, be);
    test_float_bytes!(f64, F64LE, le);
    test_float_bytes!(f64, F64BE, be);

    #[test]
    #[allow(non_snake_case)]
    fn test_I24LE_3bytes() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = I24LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24BE_3bytes() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB!
        let bytes = [allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = I24BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24LE_4bytes() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3], 0];

        let wrapped = I24LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24BE_4bytes() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [0, allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = I24BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24LE_3bytes() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = U24LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24BE_3bytes() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB!
        let bytes = [allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = U24BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24LE_4bytes() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3], 0];

        let wrapped = U24LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24BE_4bytes() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [0, allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = U24BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }
}

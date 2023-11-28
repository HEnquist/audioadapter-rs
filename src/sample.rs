use num_traits::float::Float;

/// 24 bit signed integer, little endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
#[derive(Debug)]
pub struct I24LE<U>(U);

/// 24 bit signed integer, big endian. 24 bits stored packed as as 3 bytes or padded as 4 bytes.
#[derive(Debug)]
pub struct I24BE<U>(U);

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
struct U64LE([u8; 8]);

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

/// A trait for converting a given sample type to and from floating point values
pub trait RawSample {
    /// Convert the sample value to a float in the range -1.0 .. +1.0
    fn to_scaled_float<T: Float>(&self) -> T;

    /// Convert a float in the range -1.0 .. +1.0 to a sample value
    fn from_scaled_float<T: Float>(value: T) -> Self;
}

/// A trait for converting samples stored as raw bytes into a numerical type.
pub trait BytesSample {
    type NumericType;

    const BYTES_PER_SAMPLE: usize;

    fn from_slice(bytes: &[u8]) -> Self;

    fn as_slice(&self) -> &[u8];

    /// Convert the raw bytes to a numerical value.
    /// The type of the numerical value matches the original format
    /// whenever possible, for example signed 16 bit integer samples
    /// are converted to [i16].
    /// For formats that don't have a direct match,
    /// the next larger numeric type is used.
    /// For example for 24 bit signed integers,
    /// this means [i32].
    fn to_number(&self) -> Self::NumericType;

    /// Convert a numerical value to raw bytes.
    /// The type of the numerical value matches the original format
    /// whenever possible, for example signed 16 bit integer samples
    /// are converted from [i16].
    /// For formats that don't have a direct match,
    /// the next larger numeric type is used.
    /// For example for 24 bit signed integers,
    /// this means [i32].
    fn from_number(value: Self::NumericType) -> Self;
}

macro_rules! rawsample_for_int {
    ($type:ident, $to:ident) => {
        impl RawSample for $type {
            fn to_scaled_float<T: Float>(&self) -> T {
                T::from(*self).unwrap() / (T::from($type::MAX).unwrap() + T::one())
            }

            fn from_scaled_float<T: Float>(value: T) -> Self {
                let scaled = value * (T::from($type::MAX).unwrap() + T::one());
                scaled.$to().unwrap_or(0)
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

            //TODO update
            fn from_scaled_float<T: Float>(value: T) -> Self {
                let max_ampl = (T::from($type::MAX).unwrap() + T::one()) / T::from(2).unwrap();
                let scaled = value * max_ampl + max_ampl;
                scaled.$to().unwrap_or(0)
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

            fn from_scaled_float<T: Float>(value: T) -> Self {
                value.$to().unwrap_or(0.0)
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
        i32::from_le_bytes(padded)
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

macro_rules! bytessample_for_newtype {
    ($type:ident, $newtype:ident, $from:ident, $to:ident) => {
        impl BytesSample for $newtype {
            type NumericType = $type;
            const BYTES_PER_SAMPLE: usize = std::mem::size_of::<$type>();

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

//impl RawSample for I24LE<[u8; 4]> {

impl<V> RawSample for V
where
    V: BytesSample,
    <V as BytesSample>::NumericType: RawSample,
{
    fn to_scaled_float<T: Float>(&self) -> T {
        let value = self.to_number();
        value.to_scaled_float()
    }

    fn from_scaled_float<T: Float>(value: T) -> V {
        let number = <V as BytesSample>::NumericType::from_scaled_float(value);
        V::from_number(number)
    }
}

/*
fn main() {
    let val: i32 = 1000000000;
    let fval = val.to_scaled_float::<f32>();
    //let ival = val.to_scaled_int::<i16>();
    let ival: i16 =  5;
    println!("{:?}, {}, {}", val, fval, ival);

    let bval = I24LE([1,2,3,4]);
    let bfval = bval.to_number();
    //let scaled: f64 = bval.to_scaled_float();
    //println!("{:?}, {}", bfval, scaled);
}
*/

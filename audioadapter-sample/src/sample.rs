#![allow(non_camel_case_types)]

use audio_codec_algorithms::{decode_alaw, decode_ulaw, encode_alaw, encode_ulaw};
use num_traits::{PrimInt, ToPrimitive, float::FloatCore};

// ------ 8-bit integer formats ------

/// 8 bit signed integer. Stored as 1 byte.
/// A single byte has no byte order,
/// so there are no little endian and big endian variants.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I8([u8; 1]);

/// 8 bit unsigned integer. Stored as 1 byte.
/// A single byte has no byte order,
/// so there are no little endian and big endian variants.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U8([u8; 1]);

// ------ 16-bit integer formats ------

/// 16 bit signed integer, little endian. Stored as 2 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I16_LE([u8; 2]);

/// 16 bit signed integer, big endian. Stored as 2 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I16_BE([u8; 2]);

/// 16 bit unsigned integer, little endian. Stored as 2 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U16_LE([u8; 2]);

/// 16 bit unsigned integer, big endian. Stored as 2 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U16_BE([u8; 2]);

// ----- 24-bit formats -----

/// 24 bit signed integer, little endian. Stored as 3 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_LE([u8; 3]);

/// 24 bit signed integer, little endian. Stored as 4 bytes left justified.
/// The 24 data bits are stored in the three most significant bytes,
/// while the least significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_4LJ_LE([u8; 4]);

/// 24 bit signed integer, little endian. Stored as 4 bytes right justified.
/// The 24 data bits are stored in the three least significant bytes,
/// while the most significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_4RJ_LE([u8; 4]);

/// 24 bit signed integer, big endian. Stored as 3 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_BE([u8; 3]);

/// 24 bit signed integer, big endian. Stored as 4 bytes left justified.
/// The 24 data bits are stored in the three most significant bytes,
/// while the least significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_4LJ_BE([u8; 4]);

/// 24 bit signed integer, big endian. Stored as 4 bytes right justified.
/// The 24 data bits are stored in the three least significant bytes,
/// while the most significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I24_4RJ_BE([u8; 4]);

/// 24 bit unsigned integer, little endian. Stored as 3 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_LE([u8; 3]);

/// 24 bit unsigned integer, little endian. Stored as 4 bytes left justified.
/// The 24 data bits are stored in the three most significant bytes,
/// while the least significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_4LJ_LE([u8; 4]);

/// 24 bit unsigned integer, little endian. Stored as 4 bytes right justified.
/// The 24 data bits are stored in the three least significant bytes,
/// while the most significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_4RJ_LE([u8; 4]);

/// 24 bit unsigned integer, big endian. Stored as 3 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_BE([u8; 3]);

/// 24 bit unsigned integer, big endian. Stored as 4 bytes left justified.
/// The 24 data bits are stored in the three most significant bytes,
/// while the least significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_4LJ_BE([u8; 4]);

/// 24 bit unsigned integer, big endian. Stored as 4 bytes right justified.
/// The 24 data bits are stored in the three least significant bytes,
/// while the most significant byte is unused padding.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U24_4RJ_BE([u8; 4]);

// ------ 32-bit integer formats ------

/// 32 bit signed integer, little endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I32_LE([u8; 4]);

/// 32 bit signed integer, big endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I32_BE([u8; 4]);

/// 32 bit unsigned integer, little endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U32_LE([u8; 4]);

/// 32 bit unsigned integer, big endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U32_BE([u8; 4]);

// ----- 64-bit integer formats ------

/// 64 bit signed integer, little endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I64_LE([u8; 8]);

/// 64 bit signed integer, big endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct I64_BE([u8; 8]);

/// 64 bit unsigned integer, little endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U64_LE([u8; 8]);

/// 64 bit unsigned integer, big endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct U64_BE([u8; 8]);

// ----- floating point formats -----

/// 32 bit floating point, little endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct F32_LE([u8; 4]);

/// 32 bit floating point, big endian. Stored as 4 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct F32_BE([u8; 4]);

/// 64 bit floating point, little endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct F64_LE([u8; 8]);

/// 64 bit floating point, big endian. Stored as 8 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct F64_BE([u8; 8]);

// ----- G.711 companded formats -----

/// A-law companded sample, as defined by ITU-T G.711. Stored as 1 byte.
/// A single byte has no byte order,
/// so there are no little endian and big endian variants.
///
/// A-law fits the dynamic range of a 13 bit linear value into 8 bits,
/// using a piecewise linear approximation of a logarithmic curve.
/// The closest numeric type is [i16], and the decoded values are scaled
/// to that range. The largest representable magnitude is 32256,
/// so the range does not quite reach the limits of an [i16].
/// A-law has no code for exact silence, the two smallest magnitudes
/// are +8 and -8.
///
/// Note that [i16] is wider than the 256 values this format can represent,
/// so unlike the linear formats, [`from_number`](BytesSample::from_number)
/// quantizes and a number does not survive a roundtrip unchanged.
/// Quantizing an already quantized value changes nothing further.
///
/// Note that a byte value of zero is not silence. It decodes to -5504.
/// The A-law code for the smallest positive value is `0xD5`.
///
/// This is the format used for telephony in Europe and most of the world.
/// It appears as `WAVE_FORMAT_ALAW` in wav files, as `PCMA` in RTP streams,
/// and as `SND_PCM_FORMAT_A_LAW` in ALSA.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ALAW([u8; 1]);

/// Mu-law companded sample, as defined by ITU-T G.711. Stored as 1 byte.
/// A single byte has no byte order,
/// so there are no little endian and big endian variants.
///
/// Mu-law fits the dynamic range of a 14 bit linear value into 8 bits,
/// using a piecewise linear approximation of a logarithmic curve.
/// The closest numeric type is [i16], and the decoded values are scaled
/// to that range. The largest representable magnitude is 32124,
/// so the range does not quite reach the limits of an [i16].
///
/// Note that [i16] is wider than the 256 values this format can represent,
/// so unlike the linear formats, [`from_number`](BytesSample::from_number)
/// quantizes and a number does not survive a roundtrip unchanged.
/// Quantizing an already quantized value changes nothing further.
///
/// Note that a byte value of zero is not silence. It decodes to -32124.
/// The mu-law code for silence is `0xFF`.
///
/// This is the format used for telephony in North America and Japan.
/// It appears as `WAVE_FORMAT_MULAW` in wav files, as `PCMU` in RTP streams,
/// and as `SND_PCM_FORMAT_MU_LAW` in ALSA.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct MULAW([u8; 1]);

/// Convert a float to an integer, clamp at the min and max limits of the integer.
fn to_clamped_int<T: FloatCore + ToPrimitive, U: PrimInt>(
    value: T,
    converted: Option<U>,
) -> ConversionResult<U> {
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
    fn to_scaled_float<T: FloatCore + ToPrimitive>(&self) -> T;

    /// Convert a float in the range -1.0 .. +1.0 to a sample value.
    ///
    /// For integer formats, values outside the allowed range are clipped to the
    /// nearest limit and the returned `clipped` flag is set.
    /// Floating point formats are not range-limited: values outside -1.0 .. +1.0
    /// are valid headroom, are passed through unchanged, and never set `clipped`.
    fn from_scaled_float<T: FloatCore + ToPrimitive>(value: T) -> ConversionResult<Self>;
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
    type NumericType: Copy;

    /// The number of bytes making up each sample value.
    const BYTES_PER_SAMPLE: usize;

    /// Create a sample with all bytes set to zero.
    ///
    /// This gives a correctly sized, valid value whose bytes can then be
    /// overwritten, for example via [`as_mut_slice`](Self::as_mut_slice) when
    /// reading from a stream.
    fn zero() -> Self;

    /// Create a new ByteSample from a slice of raw bytes.
    /// The slice length must be at least the number of bytes
    /// for a sample value.
    fn from_slice(bytes: &[u8]) -> Self;

    /// Return the raw bytes as a slice.
    fn as_slice(&self) -> &[u8];

    /// Return the raw bytes as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [u8];

    /// Convert the raw bytes to a numerical value.
    fn to_number(&self) -> Self::NumericType;

    /// Convert a numerical value to raw bytes.
    fn from_number(value: Self::NumericType) -> Self;
}

macro_rules! rawsample_for_int {
    ($type:ident, $to:ident) => {
        impl RawSample for $type {
            fn to_scaled_float<T: FloatCore + ToPrimitive>(&self) -> T {
                T::from(*self).unwrap() / (T::from($type::MAX).unwrap() + T::one())
            }

            fn from_scaled_float<T: FloatCore + ToPrimitive>(value: T) -> ConversionResult<Self> {
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
            fn to_scaled_float<T: FloatCore + ToPrimitive>(&self) -> T {
                let max_ampl = (T::from($type::MAX).unwrap() + T::one()) / T::from(2).unwrap();
                (T::from(*self).unwrap() - max_ampl) / max_ampl
            }

            fn from_scaled_float<T: FloatCore + ToPrimitive>(value: T) -> ConversionResult<Self> {
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
            fn to_scaled_float<T: FloatCore + ToPrimitive>(&self) -> T {
                T::from(*self).unwrap_or(T::zero())
            }

            fn from_scaled_float<T: FloatCore + ToPrimitive>(value: T) -> ConversionResult<Self> {
                // Floating point formats are not range-limited. Values outside
                // -1.0..1.0 are valid headroom and pass through unchanged, so no
                // clipping is applied and `clipped` is always false.
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

/// 24 bit signed integer, little endian, stored as 4 bytes right justified.
/// The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for I24_4RJ_LE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit signed integer, little endian, stored as 4 bytes left justified.
/// The data is in the upper 3 bytes and the least significant byte is padding.
impl BytesSample for I24_4LJ_LE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[1], self.0[2], self.0[3]];
        i32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([0, bytes[1], bytes[2], bytes[3]])
    }
}

/// 24 bit signed integer, little endian, stored as 3 bytes without padding.
impl BytesSample for I24_LE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit signed integer, big endian, stored as 4 bytes right justified.
/// The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for I24_4RJ_BE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit signed integer, big endian, stored as 4 bytes left justified.
/// The data is in the upper 3 bytes and the least significant byte is padding.
impl BytesSample for I24_4LJ_BE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[0], self.0[1], self.0[2], 0];
        i32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([bytes[0], bytes[1], bytes[2], 0])
    }
}

/// 24 bit signed integer, big endian, stored as 3 bytes without padding.
impl BytesSample for I24_BE {
    type NumericType = i32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit unsigned integer, little endian, stored as 4 bytes right justified.
/// The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for U24_4RJ_LE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit unsigned integer, little endian, stored as 4 bytes left justified.
/// The data is in the upper 3 bytes and the least significant byte is padding.
impl BytesSample for U24_4LJ_LE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [0, self.0[1], self.0[2], self.0[3]];
        u32::from_le_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_le_bytes();
        Self([0, bytes[1], bytes[2], bytes[3]])
    }
}

/// 24 bit unsigned integer, little endian, stored as 3 bytes without padding.
impl BytesSample for U24_LE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit unsigned integer, big endian, stored as 4 bytes right justified.
/// The data is in the lower 3 bytes and the most significant byte is padding.
impl BytesSample for U24_4RJ_BE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

/// 24 bit unsigned integer, big endian, stored as 4 bytes left justified.
/// The data is in the upper 3 bytes and the least significant byte is padding.
impl BytesSample for U24_4LJ_BE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..4].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }

    fn to_number(&self) -> Self::NumericType {
        let padded = [self.0[0], self.0[1], self.0[2], 0];
        u32::from_be_bytes(padded)
    }

    fn from_number(value: Self::NumericType) -> Self {
        let bytes = value.to_be_bytes();
        Self([bytes[0], bytes[1], bytes[2], 0])
    }
}

/// 24 bit unsigned integer, big endian, stored as 3 bytes without padding.
impl BytesSample for U24_BE {
    type NumericType = u32;
    const BYTES_PER_SAMPLE: usize = core::mem::size_of::<Self>();

    fn zero() -> Self {
        Self(Default::default())
    }

    fn from_slice(bytes: &[u8]) -> Self {
        Self(bytes[0..3].try_into().unwrap())
    }

    fn as_slice(&self) -> &[u8] {
        &self.0
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
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

            fn zero() -> Self {
                Self(Default::default())
            }

            fn from_slice(bytes: &[u8]) -> Self {
                Self(bytes.try_into().unwrap())
            }

            fn as_slice(&self) -> &[u8] {
                &self.0
            }

            fn as_mut_slice(&mut self) -> &mut [u8] {
                &mut self.0
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

// Single byte formats, where the endianness of the conversion is irrelevant.
bytessample_for_newtype!(i8, I8, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u8, U8, from_le_bytes, to_le_bytes);

bytessample_for_newtype!(i64, I64_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u64, U64_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i64, I64_BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u64, U64_BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(i16, I16_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u16, U16_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i16, I16_BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u16, U16_BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(i32, I32_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(u32, U32_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(i32, I32_BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(u32, U32_BE, from_be_bytes, to_be_bytes);

bytessample_for_newtype!(f32, F32_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(f32, F32_BE, from_be_bytes, to_be_bytes);
bytessample_for_newtype!(f64, F64_LE, from_le_bytes, to_le_bytes);
bytessample_for_newtype!(f64, F64_BE, from_be_bytes, to_be_bytes);

// ----- G.711 companded formats -----
//
// The conversions themselves are done by the `audio-codec-algorithms` crate.
// Its decoding tables and test vectors come from the ITU-T G.191 reference
// tools, and it verifies its encoders against them for every possible input.
//
// Both formats store the code word inverted, A-law with every other bit
// flipped and mu-law fully complemented. This dates back to the analogue
// telephone network, where it keeps the number of transitions on the line
// high enough for clock recovery.

macro_rules! bytessample_for_g711 {
    ($newtype:ident, $decode:ident, $encode:ident) => {
        impl BytesSample for $newtype {
            type NumericType = i16;
            const BYTES_PER_SAMPLE: usize = 1;

            fn zero() -> Self {
                Self(Default::default())
            }

            fn from_slice(bytes: &[u8]) -> Self {
                Self(bytes[0..1].try_into().unwrap())
            }

            fn as_slice(&self) -> &[u8] {
                &self.0
            }

            fn as_mut_slice(&mut self) -> &mut [u8] {
                &mut self.0
            }

            fn to_number(&self) -> Self::NumericType {
                $decode(self.0[0])
            }

            fn from_number(value: Self::NumericType) -> Self {
                Self([$encode(value)])
            }
        }
    };
}

bytessample_for_g711!(ALAW, decode_alaw, encode_alaw);
bytessample_for_g711!(MULAW, decode_ulaw, encode_ulaw);

impl<V> RawSample for V
where
    V: BytesSample,
    <V as BytesSample>::NumericType: RawSample,
{
    fn to_scaled_float<T: FloatCore + ToPrimitive>(&self) -> T {
        let value = self.to_number();
        value.to_scaled_float()
    }

    fn from_scaled_float<T: FloatCore + ToPrimitive>(value: T) -> ConversionResult<Self> {
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

    macro_rules! assert_conversion_eq {
        ($result:expr, $value:expr, $clipped:expr, $desc:expr) => {
            assert_eq!($result.value, $value, $desc);
            assert_eq!($result.clipped, $clipped, $desc);
        };
    }

    macro_rules! test_to_signed_int {
        ($fname:ident, $float:ty, $int:ident, $bits:expr) => {
            #[test]
            fn $fname() {
                let val: $float = 0.25;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    1 << ($bits - 3),
                    false,
                    "check +0.25"
                );
                let val: $float = -0.25;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    -1 << ($bits - 3),
                    false,
                    "check -0.25"
                );
                let val: $float = 1.1;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    $int::MAX,
                    true,
                    "clipped positive"
                );
                let val: $float = -1.1;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    $int::MIN,
                    true,
                    "clipped negative"
                );
            }
        };
    }

    macro_rules! test_to_unsigned_int {
        ($fname:ident, $float:ty, $int:ident, $bits:expr) => {
            #[test]
            fn $fname() {
                let val: $float = -0.5;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    1 << ($bits - 2),
                    false,
                    "check -0.5"
                );
                let val: $float = 0.5;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    $int::MAX - (1 << ($bits - 2)) + 1,
                    false,
                    "check 0.5"
                );
                let val: $float = 1.1;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    $int::MAX,
                    true,
                    "clipped positive"
                );
                let val: $float = -1.1;
                assert_conversion_eq!(
                    $int::from_scaled_float(val),
                    $int::MIN,
                    true,
                    "clipped negative"
                );
            }
        };
    }

    test_to_signed_int!(convert_f32_to_i8, f32, i8, 8);
    test_to_signed_int!(convert_642_to_i8, f64, i8, 8);
    test_to_signed_int!(convert_f32_to_i16, f32, i16, 16);
    test_to_signed_int!(convert_f64_to_i16, f64, i16, 16);
    test_to_signed_int!(convert_f32_to_i32, f32, i32, 32);
    test_to_signed_int!(convert_f64_to_i32, f64, i32, 32);
    test_to_signed_int!(convert_f32_to_i64, f32, i64, 64);
    test_to_signed_int!(convert_f64_to_i64, f64, i64, 64);

    test_to_unsigned_int!(convert_f32_to_u8, f32, u8, 8);
    test_to_unsigned_int!(convert_f64_to_u8, f64, u8, 8);
    test_to_unsigned_int!(convert_f32_to_u16, f32, u16, 16);
    test_to_unsigned_int!(convert_f64_to_u16, f64, u16, 16);
    test_to_unsigned_int!(convert_f32_to_u32, f32, u32, 32);
    test_to_unsigned_int!(convert_f64_to_u32, f64, u32, 32);
    test_to_unsigned_int!(convert_f32_to_u64, f32, u64, 64);
    test_to_unsigned_int!(convert_f64_to_u64, f64, u64, 64);

    macro_rules! test_from_signed_int {
        ($fname:ident, $float:ty, $int:ident, $bits:expr) => {
            #[test]
            fn $fname() {
                let val: $int = -1 << ($bits - 2);
                assert_eq!(val.to_scaled_float::<$float>(), -0.5, "check -0.5");
                let val: $int = 1 << ($bits - 2);
                assert_eq!(val.to_scaled_float::<$float>(), 0.5, "check 0.5");
                let val: $int = $int::MIN;
                assert_eq!(val.to_scaled_float::<$float>(), -1.0, "negative limit");
            }
        };
    }

    macro_rules! test_from_unsigned_int {
        ($fname:ident, $float:ty, $int:ident, $bits:expr) => {
            #[test]
            fn $fname() {
                let val: $int = 1 << ($bits - 2);
                assert_eq!(val.to_scaled_float::<$float>(), -0.5, "check -0.5");
                let val: $int = $int::MAX - (1 << ($bits - 2)) + 1;
                assert_eq!(val.to_scaled_float::<$float>(), 0.5, "check 0.5");
                let val: $int = 0;
                assert_eq!(val.to_scaled_float::<$float>(), -1.0, "negative limit");
            }
        };
    }

    test_from_signed_int!(convert_f32_from_i8, f32, i8, 8);
    test_from_signed_int!(convert_f64_from_i8, f64, i8, 8);
    test_from_signed_int!(convert_f32_from_i16, f32, i16, 16);
    test_from_signed_int!(convert_f64_from_i16, f64, i16, 16);
    test_from_signed_int!(convert_f32_from_i32, f32, i32, 32);
    test_from_signed_int!(convert_f64_from_i32, f64, i32, 32);
    test_from_signed_int!(convert_f32_from_i64, f32, i64, 64);
    test_from_signed_int!(convert_f64_from_i64, f64, i64, 64);

    test_from_unsigned_int!(convert_f32_from_u8, f32, u8, 8);
    test_from_unsigned_int!(convert_f64_from_u8, f64, u8, 8);
    test_from_unsigned_int!(convert_f32_from_u16, f32, u16, 16);
    test_from_unsigned_int!(convert_f64_from_u16, f64, u16, 16);
    test_from_unsigned_int!(convert_f32_from_u32, f32, u32, 32);
    test_from_unsigned_int!(convert_f64_from_u32, f64, u32, 32);
    test_from_unsigned_int!(convert_f32_from_u64, f32, u64, 64);
    test_from_unsigned_int!(convert_f64_from_u64, f64, u64, 64);

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
        ($fname:ident, $number:ty, $wrapper:ident, $to_bytes_fn:ident) => {
            #[test]
            #[allow(non_snake_case)]
            fn $fname() {
                let number: $number = <$number>::MAX / 5 * 4;
                let wrapped = $wrapper(number.$to_bytes_fn());
                assert_eq!(number, wrapped.to_number());
            }
        };
    }

    macro_rules! test_float_bytes {
        ($fname:ident, $number:ty, $wrapper:ident, $to_bytes_fn:ident) => {
            #[test]
            #[allow(non_snake_case)]
            fn $fname() {
                let number: $number = 12345.0;
                let wrapped = $wrapper(number.$to_bytes_fn());
                assert_eq!(number, wrapped.to_number());
            }
        };
    }

    test_simple_int_bytes!(convert_i16_from_I16_LE, i16, I16_LE, to_le_bytes);
    test_simple_int_bytes!(convert_i16_from_I16_BE, i16, I16_BE, to_be_bytes);
    test_simple_int_bytes!(convert_i32_from_I32_LE, i32, I32_LE, to_le_bytes);
    test_simple_int_bytes!(convert_i32_from_I32_BE, i32, I32_BE, to_be_bytes);
    test_simple_int_bytes!(convert_i64_from_I64_LE, i64, I64_LE, to_le_bytes);
    test_simple_int_bytes!(convert_i64_from_I64_BE, i64, I64_BE, to_be_bytes);

    test_simple_int_bytes!(convert_u16_from_U16_LE, u16, U16_LE, to_le_bytes);
    test_simple_int_bytes!(convert_u16_from_U16_BE, u16, U16_BE, to_be_bytes);
    test_simple_int_bytes!(convert_u32_from_U32_LE, u32, U32_LE, to_le_bytes);
    test_simple_int_bytes!(convert_u32_from_U32_BE, u32, U32_BE, to_be_bytes);
    test_simple_int_bytes!(convert_u64_from_U64_LE, u64, U64_LE, to_le_bytes);
    test_simple_int_bytes!(convert_u64_from_U64_BE, u64, U64_BE, to_be_bytes);

    test_float_bytes!(convert_f32_fom_F32_LE, f32, F32_LE, to_le_bytes);
    test_float_bytes!(convert_f32_fom_F32_BE, f32, F32_BE, to_be_bytes);
    test_float_bytes!(convert_f64_fom_F64_LE, f64, F64_LE, to_le_bytes);
    test_float_bytes!(convert_f64_fom_F64_BE, f64, F64_BE, to_be_bytes);

    #[test]
    #[allow(non_snake_case)]
    fn test_I8() {
        assert_eq!(I8::BYTES_PER_SAMPLE, 1);
        assert_eq!(I8::zero().to_number(), 0);
        assert_eq!(I8::from_slice(&[0x80]).to_number(), i8::MIN);

        for number in [0, 1, -1, 100, i8::MIN, i8::MAX] {
            let wrapped = I8::from_number(number);
            assert_eq!(
                wrapped.as_slice(),
                number.to_le_bytes(),
                "bytes for {number}"
            );
            assert_eq!(wrapped.to_number(), number, "roundtrip of {number}");
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U8() {
        assert_eq!(U8::BYTES_PER_SAMPLE, 1);
        assert_eq!(U8::zero().to_number(), 0);
        assert_eq!(U8::from_slice(&[0x80]).to_number(), 128);

        for number in [0, 1, 128, 200, u8::MAX] {
            let wrapped = U8::from_number(number);
            assert_eq!(
                wrapped.as_slice(),
                number.to_le_bytes(),
                "bytes for {number}"
            );
            assert_eq!(wrapped.to_number(), number, "roundtrip of {number}");
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn convert_I8_to_and_from_float() {
        assert_eq!(I8::from_slice(&[0]).to_scaled_float::<f32>(), 0.0);
        assert_eq!(I8::from_slice(&[0x80]).to_scaled_float::<f32>(), -1.0);
        assert_eq!(I8::from_slice(&[0x40]).to_scaled_float::<f32>(), 0.5);
        assert_eq!(I8::from_slice(&[0xC0]).to_scaled_float::<f32>(), -0.5);

        let converted = I8::from_scaled_float(0.5f32);
        assert_eq!(converted.value.as_slice(), [0x40]);
        assert!(!converted.clipped);

        let converted = I8::from_scaled_float(-1.0f32);
        assert_eq!(converted.value.as_slice(), [0x80]);
        assert!(!converted.clipped);

        // Values outside -1.0 .. +1.0 clip at the limits of an i8.
        let converted = I8::from_scaled_float(1.5f32);
        assert_eq!(converted.value.to_number(), i8::MAX);
        assert!(converted.clipped);

        let converted = I8::from_scaled_float(-1.5f32);
        assert_eq!(converted.value.to_number(), i8::MIN);
        assert!(converted.clipped);
    }

    #[test]
    #[allow(non_snake_case)]
    fn convert_U8_to_and_from_float() {
        // Unsigned samples are centered at 128.
        assert_eq!(U8::from_slice(&[128]).to_scaled_float::<f32>(), 0.0);
        assert_eq!(U8::from_slice(&[0]).to_scaled_float::<f32>(), -1.0);
        assert_eq!(U8::from_slice(&[192]).to_scaled_float::<f32>(), 0.5);
        assert_eq!(U8::from_slice(&[64]).to_scaled_float::<f32>(), -0.5);

        let converted = U8::from_scaled_float(0.5f32);
        assert_eq!(converted.value.as_slice(), [192]);
        assert!(!converted.clipped);

        let converted = U8::from_scaled_float(-1.0f32);
        assert_eq!(converted.value.as_slice(), [0]);
        assert!(!converted.clipped);

        // Values outside -1.0 .. +1.0 clip at the limits of a u8.
        let converted = U8::from_scaled_float(1.5f32);
        assert_eq!(converted.value.to_number(), u8::MAX);
        assert!(converted.clipped);

        let converted = U8::from_scaled_float(-1.5f32);
        assert_eq!(converted.value.to_number(), u8::MIN);
        assert!(converted.clipped);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_LE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = I24_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_BE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB!
        let bytes = [allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = I24_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_4RJ_LE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3], 0];

        let wrapped = I24_4RJ_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_4RJ_BE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [0, allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = I24_4RJ_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_4LJ_LE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Put a zero at LSB and keep the rest unchanged.
        let bytes = [0, allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = I24_4LJ_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_I24_4LJ_BE() {
        let number = i32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Put a zero at LSB and keep the rest unchanged.
        let bytes = [allbytes[0], allbytes[1], allbytes[2], 0];

        let wrapped = I24_4LJ_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_LE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = U24_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_BE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB!
        let bytes = [allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = U24_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_4RJ_LE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [allbytes[1], allbytes[2], allbytes[3], 0];

        let wrapped = U24_4RJ_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_4RJ_BE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Drop the LSB and insert padding at MSB!
        let bytes = [0, allbytes[0], allbytes[1], allbytes[2]];

        let wrapped = U24_4RJ_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_4LJ_LE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_le_bytes();
        // Little-endian stores the LSB at the smallest address.
        // Put a zero at LSB and keep the rest unchanged.
        let bytes = [0, allbytes[1], allbytes[2], allbytes[3]];

        let wrapped = U24_4LJ_LE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_U24_4LJ_BE() {
        let number = u32::MAX / 5 * 4;

        // make sure LSB is zero
        let number = number >> 8;
        let number = number << 8;

        let allbytes = number.to_be_bytes();
        // Big-endian stores the LSB at the largest address.
        // Put a zero at LSB and keep the rest unchanged.
        let bytes = [allbytes[0], allbytes[1], allbytes[2], 0];

        let wrapped = U24_4LJ_BE(bytes);
        assert_eq!(number, wrapped.to_number());
    }

    // ----- G.711 companded formats -----
    //
    // The numeric type is wider than these formats, so the roundtrip property
    // of the linear formats does not apply. A number does not survive a
    // roundtrip unless it happens to be one of the 256 representable values.
    // What does hold is that encoding is idempotent, and that every code word
    // survives a roundtrip through its numeric value.

    #[test]
    #[allow(non_snake_case)]
    fn test_ALAW() {
        assert_eq!(ALAW::BYTES_PER_SAMPLE, 1);

        // A-law has no code for exact silence, the smallest magnitudes are +-8.
        assert_eq!(ALAW::from_slice(&[0xd5]).to_number(), 8);
        assert_eq!(ALAW::from_slice(&[0x55]).to_number(), -8);
        assert_eq!(ALAW::from_number(0).as_slice(), [0xd5]);

        // The largest representable magnitudes.
        assert_eq!(ALAW::from_slice(&[0xaa]).to_number(), 32256);
        assert_eq!(ALAW::from_slice(&[0x2a]).to_number(), -32256);

        // An all zero byte is a valid code word, but it is not silence.
        assert_eq!(ALAW::zero().to_number(), -5504);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_MULAW() {
        assert_eq!(MULAW::BYTES_PER_SAMPLE, 1);

        // Mu-law has two codes for silence, one per sign.
        assert_eq!(MULAW::from_slice(&[0xff]).to_number(), 0);
        assert_eq!(MULAW::from_slice(&[0x7f]).to_number(), 0);
        assert_eq!(MULAW::from_number(0).as_slice(), [0xff]);

        // The largest representable magnitudes. Note that the sign bit means
        // the opposite of what it means in A-law.
        assert_eq!(MULAW::from_slice(&[0x80]).to_number(), 32124);
        assert_eq!(MULAW::from_slice(&[0x00]).to_number(), -32124);

        // An all zero byte is a valid code word, but it is not silence.
        assert_eq!(MULAW::zero().to_number(), -32124);
    }

    macro_rules! test_g711_roundtrips {
        ($name:ident, $type:ident, $maxmagnitude:expr, $aliases:expr) => {
            #[test]
            #[allow(non_snake_case)]
            fn $name() {
                // Every code word survives a roundtrip via its numeric value,
                // apart from any duplicate encoding of the same value.
                for byte in 0..=u8::MAX {
                    if $aliases.contains(&byte) {
                        continue;
                    }
                    let number = $type::from_slice(&[byte]).to_number();
                    assert_eq!(
                        $type::from_number(number).as_slice(),
                        [byte],
                        "code word {byte:#04x} decoded to {number}"
                    );
                }

                // Encoding is idempotent. Quantizing an already quantized
                // value leaves it unchanged.
                for number in (i16::MIN..=i16::MAX).step_by(7) {
                    let once = $type::from_number(number).to_number();
                    let twice = $type::from_number(once).to_number();
                    assert_eq!(once, twice, "quantizing {number} is not stable");
                }

                // The decoded values never exceed the documented magnitude,
                // and the format is symmetric around zero.
                for byte in 0..=u8::MAX {
                    let number = $type::from_slice(&[byte]).to_number();
                    assert!(
                        number.abs() <= $maxmagnitude,
                        "code word {byte:#04x} decoded to {number}"
                    );
                    let mirrored = $type::from_slice(&[byte ^ 0x80]).to_number();
                    assert_eq!(number, -mirrored, "code word {byte:#04x} is not symmetric");
                }
            }
        };
    }

    // A-law encodes every value exactly once. Mu-law has two codes for zero,
    // and `0xff` is the one that encoding produces.
    test_g711_roundtrips!(roundtrip_ALAW, ALAW, 32256, []);
    test_g711_roundtrips!(roundtrip_MULAW, MULAW, 32124, [0x7f]);

    #[test]
    #[allow(non_snake_case)]
    fn convert_ALAW_to_and_from_float() {
        assert_eq!(
            ALAW::from_slice(&[0xaa]).to_scaled_float::<f32>(),
            32256.0 / 32768.0
        );
        assert_eq!(
            ALAW::from_slice(&[0x2a]).to_scaled_float::<f32>(),
            -32256.0 / 32768.0
        );
        assert!(ALAW::from_slice(&[0xd5]).to_scaled_float::<f32>().abs() < 0.001);

        // Values outside -1.0 .. +1.0 clip, and land on the largest magnitude.
        let converted = ALAW::from_scaled_float(1.5f32);
        assert_eq!(converted.value.to_number(), 32256);
        assert!(converted.clipped);

        let converted = ALAW::from_scaled_float(-1.5f32);
        assert_eq!(converted.value.to_number(), -32256);
        assert!(converted.clipped);
    }

    #[test]
    #[allow(non_snake_case)]
    fn convert_MULAW_to_and_from_float() {
        assert_eq!(
            MULAW::from_slice(&[0x80]).to_scaled_float::<f32>(),
            32124.0 / 32768.0
        );
        assert_eq!(
            MULAW::from_slice(&[0x00]).to_scaled_float::<f32>(),
            -32124.0 / 32768.0
        );
        assert_eq!(MULAW::from_slice(&[0xff]).to_scaled_float::<f32>(), 0.0);

        // Values outside -1.0 .. +1.0 clip, and land on the largest magnitude.
        let converted = MULAW::from_scaled_float(1.5f32);
        assert_eq!(converted.value.to_number(), 32124);
        assert!(converted.clipped);

        let converted = MULAW::from_scaled_float(-1.5f32);
        assert_eq!(converted.value.to_number(), -32124);
        assert!(converted.clipped);
    }

    /// Check that the companding curve really is logarithmic, by verifying that
    /// the error stays proportional to the value instead of being a fixed step.
    ///
    /// This only holds above the bottom of the range. Both curves have a linear
    /// section around zero, where the step size stops shrinking and the relative
    /// error grows without bound. That part is checked as an absolute error.
    macro_rules! test_g711_accuracy {
        ($name:ident, $type:ident, $smallerror:expr) => {
            #[test]
            #[allow(non_snake_case)]
            fn $name() {
                for value in (i16::MIN..=i16::MAX).step_by(3) {
                    let encoded = $type::from_number(value).to_number();
                    let error = (i32::from(encoded) - i32::from(value)).abs();
                    if value.unsigned_abs() >= 1000 {
                        let relative = f64::from(error) / f64::from(value.unsigned_abs());
                        assert!(
                            relative < 0.05,
                            "relative error {relative} at {value} is too large, got {encoded}"
                        );
                    } else {
                        assert!(
                            error <= $smallerror,
                            "error {error} at {value} is too large, got {encoded}"
                        );
                    }
                }
            }
        };
    }

    // The measured worst cases are 16 and 32, so these are the exact bounds.
    test_g711_accuracy!(accuracy_ALAW, ALAW, 16);
    test_g711_accuracy!(accuracy_MULAW, MULAW, 32);
}

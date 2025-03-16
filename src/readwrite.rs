use crate::sample::*;
use num_traits::Float;
use std::io;

/// A trait that extends [std::io::Read] with methods for reading samples directly.
pub trait ReadSamples: io::Read {
    /// Read a single sample and return it as a numeric type.
    ///
    /// This method reads a chunk of bytes from the underlying reader,
    /// interprets it as a sample of type `T`, and returns the sample
    /// converted to its associated `NumericType`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing the `BytesSample` trait, defining the format of the sample to read.
    ///
    /// # Returns
    ///
    /// * `io::Result<T::NumericType>`: The sample as a number, or an error if reading failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error.
    /// * The number of bytes read is not sufficient to represent a complete sample.
    fn read_number<T: BytesSample>(&mut self) -> io::Result<T::NumericType> {
        let mut buf = [0; 8];
        let sliced_buf = &mut buf[..T::BYTES_PER_SAMPLE];
        self.read_exact(sliced_buf)?;
        Ok(T::from_slice(sliced_buf).to_number())
    }

    /// Read a single sample and convert it to a floating-point number.
    ///
    /// This method reads a chunk of bytes from the underlying reader,
    /// interprets it as a sample of type `T`, and returns the sample
    /// converted to a floating-point number of type `U`. The conversion
    /// uses the `to_scaled_float` method from the `RawSample` trait,
    /// ensuring that the float is scaled between -1.0 and 1.0.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing both `RawSample` and `BytesSample`, defining the format of the sample to read.
    /// * `U`: A floating-point type implementing `Float`, representing the desired output format.
    ///
    /// # Returns
    ///
    /// * `io::Result<U>`: The converted sample as a float, or an error if reading failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error.
    /// * The number of bytes read is not sufficient to represent a complete sample.
    fn read_converted<T: RawSample + BytesSample, U: Float>(&mut self) -> io::Result<U> {
        let mut buf = [0; 8];
        let sliced_buf = &mut buf[..T::BYTES_PER_SAMPLE];
        self.read_exact(sliced_buf)?;
        Ok(T::from_slice(sliced_buf).to_scaled_float::<U>())
    }

    /// Read multiple samples and store them as numeric types in a provided buffer.
    ///
    /// This method reads a sequence of samples from the underlying reader and
    /// stores them in the provided buffer `buf`. Each sample is read and
    /// interpreted as a type `T`, then converted to its associated `NumericType`
    /// before being stored in the buffer.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing the `BytesSample` trait, defining the format of the samples to read.
    ///
    /// # Arguments
    ///
    /// * `buf`: A mutable slice where the samples will be stored. The length of the slice determines how many samples are read.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>`: Ok(()) if all samples were read successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error.
    /// * The number of bytes read is not sufficient to represent a complete sample.
    /// * The end of the reader is reached before all samples have been read.
    fn read_numbers_exact<T: BytesSample>(&mut self, buf: &mut [T::NumericType]) -> io::Result<()> {
        for sample in buf.iter_mut() {
            *sample = self.read_number::<T>()?;
        }
        Ok(())
    }

    /// Read multiple samples, convert them to floats, and store them in a provided buffer.
    ///
    /// This method reads a sequence of samples from the underlying reader,
    /// converts each sample to a floating-point number of type `U`, and
    /// stores the results in the provided buffer `buf`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing both `RawSample` and `BytesSample`, defining the format of the samples to read.
    /// * `U`: A floating-point type implementing `Float`, representing the desired output format.
    ///
    /// # Arguments
    ///
    /// * `buf`: A mutable slice where the converted samples will be stored. The length of the slice determines how many samples are read.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>`: Ok(()) if all samples were read and converted successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error.
    /// * The number of bytes read is not sufficient to represent a complete sample.
    /// * The end of the reader is reached before all samples have been read.
    fn read_converted_exact<T: RawSample + BytesSample, U: Float>(
        &mut self,
        buf: &mut [U],
    ) -> io::Result<()> {
        for sample in buf.iter_mut() {
            *sample = self.read_converted::<T, U>()?;
        }
        Ok(())
    }

    /// Read samples until the end of the stream, storing them as numeric types in a vector.
    ///
    /// This method reads samples from the underlying reader until an error is encountered
    /// (typically, the end of the stream). Each sample is read and interpreted as
    /// a type `T`, then converted to its associated `NumericType` before being
    /// appended to the provided vector `buf`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing the `BytesSample` trait, defining the format of the samples to read.
    ///
    /// # Arguments
    ///
    /// * `buf`: A mutable vector where the samples will be appended.
    ///
    /// # Returns
    ///
    /// * `io::Result<usize>`: The number of samples read, or an error if reading failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error (except for EOF).
    /// * The number of bytes read is not sufficient to represent a complete sample.
    fn read_numbers_to_end<T: BytesSample>(
        &mut self,
        buf: &mut Vec<T::NumericType>,
    ) -> io::Result<usize> {
        let mut count = 0;
        while let Ok(sample) = self.read_number::<T>() {
            buf.push(sample);
            count += 1;
        }
        Ok(count)
    }

    /// Read samples until the end of the stream, converting them to floats, and store in a vector.
    ///
    /// This method reads samples from the underlying reader until an error is encountered
    /// (typically, the end of the stream). Each sample is read, converted to a
    /// floating-point number of type `U`, and appended to the provided vector `buf`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing both `RawSample` and `BytesSample`, defining the format of the samples to read.
    /// * `U`: A floating-point type implementing `Float`, representing the desired output format.
    ///
    /// # Arguments
    ///
    /// * `buf`: A mutable vector where the converted samples will be appended.
    ///
    /// # Returns
    ///
    /// * `io::Result<usize>`: The number of samples read and converted, or an error if reading failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying reader returns an error (except for EOF).
    /// * The number of bytes read is not sufficient to represent a complete sample.
    fn read_converted_to_end<T: RawSample + BytesSample, U: Float>(
        &mut self,
        buf: &mut Vec<U>,
    ) -> io::Result<usize> {
        let mut count = 0;
        while let Ok(sample) = self.read_converted::<T, U>() {
            buf.push(sample);
            count += 1;
        }
        Ok(count)
    }
}

impl<R: io::Read + ?Sized> ReadSamples for R {}

/// A trait that extends [std::io::Write] with methods for writing samples directly.
pub trait WriteSamples: io::Write {
    /// Write a single sample from a numeric type to the underlying writer.
    ///
    /// This method takes a sample represented by its `NumericType`,
    /// converts it to a raw byte representation using the provided `T`,
    /// and writes it to the underlying writer.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing the `BytesSample` trait, defining the format of the sample to write.
    ///
    /// # Arguments
    ///
    /// * `value`: The sample to write.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>`: Ok(()) if the sample was written successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying writer returns an error.
    fn write_number<T: BytesSample>(&mut self, value: T::NumericType) -> io::Result<()> {
        self.write_all(T::from_number(value).as_slice())
    }

    /// Write a single converted sample to the underlying writer.
    ///
    /// This method takes a floating-point sample of type `U`, converts it to
    /// the raw byte representation of type `T` and then writes it to the underlying writer.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing both `RawSample` and `BytesSample`, defining the format of the sample to write.
    /// * `U`: A floating-point type implementing `Float`, representing the sample to write.
    ///
    /// # Arguments
    ///
    /// * `value`: The sample to write.
    ///
    /// # Returns
    ///
    /// * `io::Result<bool>`: Ok(true) if the value was clipped during conversion, Ok(false) otherwise.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying writer returns an error.
    fn write_converted<T: RawSample + BytesSample, U: Float>(
        &mut self,
        value: U,
    ) -> io::Result<bool> {
        let converted = T::from_scaled_float(value);
        self.write_all(converted.value.as_slice())?;
        Ok(converted.clipped)
    }

    /// Write multiple samples from a numeric slice to the underlying writer.
    ///
    /// This method takes a slice of samples represented by their `NumericType`,
    /// converts each sample to its raw byte representation using the provided `T`,
    /// and writes them to the underlying writer.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing the `BytesSample` trait, defining the format of the samples to write.
    ///
    /// # Arguments
    ///
    /// * `values`: The samples to write.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>`: Ok(()) if all samples were written successfully.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying writer returns an error.
    fn write_all_numbers<T: BytesSample>(&mut self, values: &[T::NumericType]) -> io::Result<()> {
        for value in values {
            self.write_number::<T>(*value)?;
        }
        Ok(())
    }

    /// Write multiple converted samples from a float slice to the underlying writer.
    ///
    /// This method takes a slice of floating-point samples of type `U`, converts each sample
    /// to its raw byte representation of type `T`, and then writes them to the underlying writer.
    ///
    /// # Type Parameters
    ///
    /// * `T`: A type implementing both `RawSample` and `BytesSample`, defining the format of the samples to write.
    /// * `U`: A floating-point type implementing `Float`, representing the samples to write.
    ///
    /// # Arguments
    ///
    /// * `values`: The samples to write.
    ///
    /// # Returns
    ///
    /// * `io::Result<usize>`: The number of samples that were clipped during conversion.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * The underlying writer returns an error.
    fn write_all_converted<T: RawSample + BytesSample, U: Float>(
        &mut self,
        values: &[U],
    ) -> io::Result<usize> {
        let mut nbr_clipped = 0;
        for value in values {
            let clipped = self.write_converted::<T, U>(*value)?;
            if clipped {
                nbr_clipped += 1;
            }
        }
        Ok(nbr_clipped)
    }
}

impl<W: io::Write + ?Sized> WriteSamples for W {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readwrite::ReadSamples;
    use crate::readwrite::WriteSamples;

    #[test]
    fn test_read_number_i16() {
        let data: Vec<u8> = vec![0, 1, 2, 3];
        let mut slice = &data[..];
        assert_eq!(slice.read_number::<I16LE>().unwrap(), 256);
        assert_eq!(slice.read_number::<I16LE>().unwrap(), 3 * 256 + 2);
        assert!(slice.read_number::<I16LE>().is_err());
    }

    #[test]
    fn test_read_converted_i16() {
        let data: Vec<u8> = vec![0, 64, 0, 32];
        let mut slice = &data[..];
        assert_eq!(slice.read_converted::<I16LE, f32>().unwrap(), 0.5);
        assert_eq!(slice.read_converted::<I16LE, f32>().unwrap(), 0.25);
        assert!(slice.read_converted::<I16LE, f32>().is_err());
    }

    #[test]
    fn test_read_number_exact_i16() {
        let data: Vec<u8> = vec![0, 1, 2, 3];
        let mut slice = &data[..];
        let mut buf = [0; 2];
        slice.read_numbers_exact::<I16LE>(&mut buf).unwrap();
        assert_eq!(buf, [256, 3 * 256 + 2]);
        assert!(slice.read_numbers_exact::<I16LE>(&mut buf).is_err());
    }

    #[test]
    fn test_read_converted_exact_i16() {
        let data: Vec<u8> = vec![0, 64, 0, 32];
        let mut slice = &data[..];
        let mut buf = [0.0; 2];
        slice.read_converted_exact::<I16LE, f32>(&mut buf).unwrap();
        assert_eq!(buf, [0.5, 0.25]);
        assert!(slice.read_converted_exact::<I16LE, f32>(&mut buf).is_err());
    }

    #[test]
    fn test_read_numbers_to_end_i16() {
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut slice = &data[..];
        let mut buf = Vec::new();
        slice.read_numbers_to_end::<I16LE>(&mut buf).unwrap();
        assert_eq!(buf, [256, 3 * 256 + 2, 5 * 256 + 4, 7 * 256 + 6]);
    }

    #[test]
    fn test_read_converted_to_end_i16() {
        let data: Vec<u8> = vec![0, 64, 0, 32, 0, 16, 0, 8];
        let mut slice = &data[..];
        let mut buf = Vec::new();
        slice.read_converted_to_end::<I16LE, f32>(&mut buf).unwrap();
        assert_eq!(buf, [0.5, 0.25, 0.125, 0.0625]);
    }

    #[test]
    fn test_write_number_i16() {
        let mut buf = Vec::new();
        buf.write_number::<I16LE>(256).unwrap();
        buf.write_number::<I16LE>(3 * 256 + 2).unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);
    }

    #[test]
    fn test_write_converted_i16() {
        let mut buf = Vec::new();
        buf.write_converted::<I16LE, f32>(0.5).unwrap();
        buf.write_converted::<I16LE, f32>(0.25).unwrap();
        assert_eq!(buf, [0, 64, 0, 32]);
    }

    #[test]
    fn test_write_all_numbers_i16() {
        let mut buf = Vec::new();
        buf.write_all_numbers::<I16LE>(&[256, 3 * 256 + 2]).unwrap();
        assert_eq!(buf, [0, 1, 2, 3]);
    }

    #[test]
    fn test_write_all_converted_i16() {
        let mut buf = Vec::new();
        buf.write_all_converted::<I16LE, f32>(&[0.5, 0.25]).unwrap();
        assert_eq!(buf, [0, 64, 0, 32]);
    }
}

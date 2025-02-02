use std::{fs::File, path::Path};

use anyhow::Result;
use memmap2::Mmap;

use crate::imu_data::ImuData;

pub struct XrImuReader {
    _file: File,
    shm: Mmap,
}

impl XrImuReader {
    pub fn new(file_name: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(file_name)?;
        let mmap = unsafe { Mmap::map(&file)? };

        Ok(Self {
            _file: file,
            shm: mmap,
        })
    }

    #[cfg(test)]
    fn read_all_data(&self) -> Result<Vec<u8>> {
        use std::io::Read;
        use std::ops::Deref;

        let mut v = Vec::new();
        self.shm.deref().read_to_end(&mut v)?;

        Ok(v)
    }

    fn read(&self, offset: usize, length: usize) -> &[u8] {
        let b = &self.shm[offset..offset + length];

        debug_assert_eq!(b.len(), length);

        b
    }

    pub fn read_imu_data(&self) -> Result<ImuData> {
        let raw = self.read(0, size_of::<ImuData>());
        let raw_array: [u8; size_of::<ImuData>()] = raw.try_into()?;

        Ok(ImuData::from(raw_array))
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use crate::imu_reader::XrImuReader;
    use anyhow::{bail, Result};

    #[test]
    fn test_imu_data() -> Result<()> {
        let imu_reader = XrImuReader::new("/tmp/shader_runtime_imu_quat_data")?;

        let mut read_data = false;

        for i in 0..10 {
            let s = imu_reader.read_all_data()?;

            if !s.is_empty() {
                read_data = true;
            }

            println!("Read {i}: {s:?}");

            thread::sleep(Duration::from_secs(1));
        }

        if !read_data {
            bail!("failed to read any data");
        }

        Ok(())
    }
}

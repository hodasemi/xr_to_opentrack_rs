use std::fs::File;

use anyhow::Result;
use memmap2::Mmap;

use crate::imu_data::ImuData;

pub struct XrImuReader {
    _file: File,
    shm: Mmap,
}

impl XrImuReader {
    pub fn new(file_name: &str) -> Result<Self> {
        let file = File::open(file_name)?;
        let mmap = unsafe { Mmap::map(&file)? };

        Ok(Self {
            _file: file,
            shm: mmap,
        })
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

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;
use std::sync::Arc;

pub struct ArcFileReader {
    file: Arc<File>,
    start_offset: u64,
    current_pos: u64,
}

impl ArcFileReader {
    pub fn new(file: Arc<File>, start_offset: u64) -> Self {
        Self {
            file,
            start_offset,
            current_pos: 0,
        }
    }
}

impl Read for ArcFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let absolute_offset = self.start_offset + self.current_pos;

        #[cfg(unix)]
        let bytes_read = self.file.read_at(buf, absolute_offset)?;
        #[cfg(windows)]
        let bytes_read = self.file.seek_read(buf, absolute_offset)?;
        #[cfg(not(any(unix, windows)))]
        let bytes_read = {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "File reading is not supported on this platform",
            ));
        };

        self.current_pos += bytes_read as u64;

        Ok(bytes_read)
    }
}

impl Seek for ArcFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "SeekFrom::End is not supported",
                ));
            }
            SeekFrom::Current(p) => self.current_pos as i64 + p,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid seek to a negative position",
            ));
        }

        self.current_pos = new_pos as u64;
        Ok(self.current_pos)
    }
}

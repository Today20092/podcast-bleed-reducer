//! Read-only handles recover bounded transient failures without skipping bytes.
use std::{
    fs::{File, Metadata},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug)]
struct Failure {
    operation: &'static str,
    path: PathBuf,
    source: io::Error,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guidance = match self.source.raw_os_error() {
            Some(19) if cfg!(windows) => {
                "The destination is write-protected. Choose a writable export folder; the source card may stay locked."
            }
            Some(112) if cfg!(windows) => {
                "Free space on this drive or choose another destination, then retry."
            }
            _ => match self.source.kind() {
                io::ErrorKind::PermissionDenied => {
                    "Check folder permissions or choose a writable destination. Reading originals does not require write access."
                }
                io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => {
                    "Free space on this drive or choose another destination, then retry."
                }
                io::ErrorKind::ReadOnlyFilesystem => {
                    "The destination is read-only. Choose a writable export folder; the source card may stay locked."
                }
                io::ErrorKind::FileTooLarge => {
                    "This filesystem cannot store the output file size. Choose a destination supporting larger files or export a shorter recording."
                }
                io::ErrorKind::NotFound | io::ErrorKind::NotConnected => {
                    "Reconnect the drive shown above and retry. Relink a moved recording, or choose another export destination."
                }
                _ if transient(&self.source) => {
                    "Check the card/drive connection and retry. Edits are retained; incomplete output is not accepted."
                }
                _ => {
                    "Check this location and retry. Existing recordings and completed outputs are retained."
                }
            },
        };
        write!(
            f,
            "Could not {} '{}': {}. {}",
            self.operation,
            self.path.display(),
            self.source,
            guidance
        )
    }
}
impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
pub(crate) fn context(operation: &'static str, path: &Path, source: io::Error) -> io::Error {
    io::Error::new(
        if source.kind() == io::ErrorKind::Interrupted {
            io::ErrorKind::Other
        } else {
            source.kind()
        },
        Failure {
            operation,
            path: path.into(),
            source,
        },
    )
}

fn transient(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::Interrupted
            | io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::NotConnected
    ) || (cfg!(windows) && matches!(error.raw_os_error(), Some(21 | 55 | 64 | 121 | 1006 | 1167)))
}

#[derive(PartialEq, Eq)]
struct Stamp {
    len: u64,
    modified: Option<SystemTime>,
}
impl Stamp {
    fn read(metadata: Metadata) -> Self {
        Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        }
    }
}

pub(crate) struct SourceReader {
    file: File,
    path: PathBuf,
    stamp: Stamp,
    position: u64,
    retries: u8,
    #[cfg(test)]
    reads: u64,
}
impl SourceReader {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        // An absolute path prevents a reconnect from resolving against a changed working directory.
        let path = std::path::absolute(path)?;
        let mut retries = 0;
        let file = loop {
            match File::open(&path) {
                Ok(file) => break file,
                Err(e) if transient(&e) && retries < 2 => {
                    retries += 1;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => return Err(context("open file for reading", &path, e)),
            }
        };
        let stamp = Stamp::read(
            file.metadata()
                .map_err(|e| context("inspect file", &path, e))?,
        );
        Ok(Self {
            file,
            path,
            stamp,
            position: 0,
            retries,
            #[cfg(test)]
            reads: 0,
        })
    }
    #[cfg(test)]
    pub(crate) fn read_count(&self) -> u64 {
        self.reads
    }
    pub(crate) fn metadata(&self) -> io::Result<Metadata> {
        self.file
            .metadata()
            .map_err(|e| context("inspect file", &self.path, e))
    }
    fn reopen(&mut self) -> io::Result<()> {
        let mut file = File::open(&self.path)?;
        if Stamp::read(file.metadata()?) != self.stamp {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Recording changed during reconnect; relink and review before retrying.",
            ));
        }
        file.seek(SeekFrom::Start(self.position))?;
        self.file = file;
        Ok(())
    }
    fn perform<T>(
        &mut self,
        operation: &'static str,
        mut action: impl FnMut(&mut File) -> io::Result<T>,
    ) -> io::Result<T> {
        loop {
            match action(&mut self.file) {
                Ok(value) => return Ok(value),
                Err(mut error) => loop {
                    if !transient(&error) || self.retries >= 2 {
                        return Err(context(operation, &self.path, error));
                    }
                    self.retries += 1;
                    // At most 100 ms of reconnect backoff per reader; callers retain cancellation checks.
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    match self.reopen() {
                        Ok(()) => break,
                        Err(e) => error = e,
                    }
                },
            }
        }
    }
}
impl Read for SourceReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        #[cfg(test)]
        {
            self.reads += 1;
        }
        let count = self.perform("read file", |file| file.read(buffer))?;
        self.position += count as u64;
        Ok(count)
    }
}
impl Seek for SourceReader {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Start(position) => Some(position),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset),
            SeekFrom::End(offset) => self.stamp.len.checked_add_signed(offset),
        }
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid audio seek position")
        })?;
        let position = self.perform("seek file", |file| file.seek(SeekFrom::Start(target)))?;
        self.position = position;
        Ok(position)
    }
}
impl symphonia::core::io::MediaSource for SourceReader {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.stamp.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transient_read_reopens_at_confirmed_position_without_missing_or_duplicate_bytes() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("audio");
        std::fs::write(&path, b"abcdefgh").unwrap();
        let mut reader = SourceReader::open(&path).unwrap();
        let mut first = [0; 3];
        reader.read_exact(&mut first).unwrap();
        let mut attempts = 0;
        let mut rest = [0; 5];
        let count = reader
            .perform("read audio", |file| {
                attempts += 1;
                if attempts == 1 {
                    file.seek(SeekFrom::End(0))?;
                    return Err(io::Error::from(io::ErrorKind::TimedOut));
                }
                file.read(&mut rest)
            })
            .unwrap();
        assert_eq!(count, 5);
        assert_eq!(&first, b"abc");
        assert_eq!(&rest, b"defgh");
        assert_eq!(attempts, 2);
        assert_eq!(reader.retries, 1);
        assert_eq!(std::fs::read(&path).unwrap(), b"abcdefgh");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retries_are_bounded_and_changed_media_and_permanent_errors_are_rejected() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("audio");
        std::fs::write(&path, b"original").unwrap();
        let mut reader = SourceReader::open(&path).unwrap();
        let mut attempts = 0;
        let error = reader
            .perform::<()>("read audio", |_| {
                attempts += 1;
                Err(io::Error::from(io::ErrorKind::TimedOut))
            })
            .unwrap_err()
            .to_string();
        assert_eq!(attempts, 3);
        assert!(error.contains("read audio") && error.contains("audio") && error.contains("retry"));
        let mut reader = SourceReader::open(&path).unwrap();
        assert!(
            reader
                .perform::<()>("read audio", |_| {
                    std::fs::write(&path, b"different recording").unwrap();
                    Err(io::Error::from(io::ErrorKind::TimedOut))
                })
                .unwrap_err()
                .to_string()
                .contains("Recording changed")
        );
        let mut reader = SourceReader::open(&path).unwrap();
        let error = reader
            .perform::<()>("read audio", |_| {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            })
            .unwrap_err();
        assert_eq!(reader.retries, 0);
        assert!(error.to_string().contains("permissions"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn write_protection_and_full_disk_offer_destination_guidance() {
        let path = Path::new("export.wav");
        assert!(
            context(
                "write export",
                path,
                io::Error::from(io::ErrorKind::StorageFull)
            )
            .to_string()
            .contains("Free space")
        );
        #[cfg(windows)]
        assert!(
            context("write export", path, io::Error::from_raw_os_error(19))
                .to_string()
                .contains("write-protected")
        );
    }

    #[test]
    fn interrupted_reads_stop_after_the_budget_instead_of_read_exact_retrying_forever() {
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("audio");
        std::fs::write(&path, b"original").unwrap();
        let mut reader = SourceReader::open(&path).unwrap();
        let error = reader
            .perform::<()>("read audio", |_| {
                Err(io::Error::from(io::ErrorKind::Interrupted))
            })
            .unwrap_err();
        assert_eq!(reader.retries, 2);
        assert_ne!(error.kind(), io::ErrorKind::Interrupted);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_1006_uses_fresh_handles_and_other_os_errors_are_not_retried() {
        assert!(transient(&io::Error::from_raw_os_error(1006)));
        for code in [5, 19, 23, 112] {
            assert!(!transient(&io::Error::from_raw_os_error(code)));
        }
        let root = crate::audio_io::multichannel_tests::root();
        let path = root.join("audio");
        std::fs::write(&path, b"original").unwrap();
        let mut reader = SourceReader::open(&path).unwrap();
        let mut attempts = 0;
        reader
            .perform("read audio", |_| {
                attempts += 1;
                if attempts == 1 {
                    Err(io::Error::from_raw_os_error(1006))
                } else {
                    Ok(())
                }
            })
            .unwrap();
        assert_eq!(attempts, 2);
        std::fs::remove_dir_all(root).unwrap();
    }
}

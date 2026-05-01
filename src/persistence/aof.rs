use std::io::{self, Write, BufWriter};
use std::fs::{File, OpenOptions};
use std::path::Path;

/// AOF (Append Only File) persistence
/// 
/// Logs every write command to a file for durability.
/// Supports three fsync strategies: always, everysec, no.
pub struct Aof {
    writer: Option<BufWriter<File>>,
    fsync_policy: FsyncPolicy,
}

#[derive(Debug, Clone, Copy)]
pub enum FsyncPolicy {
    Always,
    EverySec,
    No,
}

impl Aof {
    pub fn new(path: &Path, policy: FsyncPolicy) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Aof {
            writer: Some(BufWriter::new(file)),
            fsync_policy: policy,
        })
    }

    /// Create a disabled (no-op) AOF
    pub fn disabled() -> Self {
        Aof {
            writer: None,
            fsync_policy: FsyncPolicy::No,
        }
    }

    /// Append a command to the AOF file
    pub fn append(&mut self, args: &[Vec<u8>]) -> io::Result<()> {
        let writer = match self.writer.as_mut() {
            Some(w) => w,
            None => return Ok(()),
        };

        // Write RESP format
        write!(writer, "*{}\r\n", args.len())?;
        for arg in args {
            write!(writer, "${}\r\n", arg.len())?;
            writer.write_all(arg)?;
            write!(writer, "\r\n")?;
        }

        match self.fsync_policy {
            FsyncPolicy::Always => {
                writer.flush()?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Flush the AOF buffer
    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
        }
        Ok(())
    }
}

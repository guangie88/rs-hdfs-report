extern crate chrono;
extern crate json_collection;
extern crate which;

use failure::{Fail, ResultExt};
use regex::Regex;
use self::json_collection::{Storage, StorageBuilder};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use super::error::{ErrorKind, RegexCaptureError, Result};
use super::util::extract_output_stdout_str;

const HDFS: &str = "hdfs";
const DFS: &str = "dfs";

#[derive(Debug)]
pub struct Hdfs {
    p: PathBuf,
}

impl Hdfs {
    pub fn new() -> Result<Hdfs> {
        let p = which::which(HDFS).context(ErrorKind::HdfsNotAvailable)?;
        Ok(Hdfs::with_path(p))
    }

    pub fn with_path(p: PathBuf) -> Hdfs {
        Hdfs { p }
    }

    pub fn df(&self, path: &str) -> Result<Storage> {
        let df = Command::new(&self.p)
            .args(&[DFS, "-df", path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(ErrorKind::HdfsDfCmd)?;

        let df_str = extract_output_stdout_str(df)?;

        lazy_static! {
            static ref CAP_LINE_RE: Regex = Regex::new(
                r"^Filesystem\s+Size\s+Used\s+Available\s+Use%\s+(?P<v>.+)\s"
            ).unwrap();
            static ref VALUES_RE: Regex = Regex::new(
                r"^(?P<fs>\S+)\s+(?P<s>\S+)\s+(?P<u>\S+)\s+\S+\s+\S+%.*$"
            ).unwrap();
        }

        let caps = CAP_LINE_RE.captures(&df_str).ok_or_else(|| {
            RegexCaptureError::new(&CAP_LINE_RE, df_str.to_owned())
                .context(ErrorKind::RegexInitialHdfsDfCap)
        })?;

        let value_str = &caps["v"];

        let values = VALUES_RE.captures(value_str).ok_or_else(|| {
            RegexCaptureError::new(&VALUES_RE, value_str.to_owned())
                .context(ErrorKind::RegexHdfsDfValuesCap)
        })?;

        let filesystem = values["fs"].to_owned();

        let size = values["s"]
            .parse::<u64>()
            .context(ErrorKind::ParseHdfsDfSizeValue)?;

        let used = values["u"]
            .parse::<u64>()
            .context(ErrorKind::ParseHdfsDfUsedValue)?;

        Ok(StorageBuilder::default()
            .path(filesystem)
            .used(used)
            .capacity(size)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

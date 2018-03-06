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

fn parse_df<S>(df_str: S) -> Result<Storage>
where
    S: AsRef<str>,
{
    let df_str = df_str.as_ref();

    // ignore the total size / capacity because that is the filesystem caps
    // look only at the space used and available for HDFS only
    lazy_static! {
        static ref CAP_LINE_RE: Regex = Regex::new(
            r"^Filesystem\s+Size\s+Used\s+Available\s+Use%\s+(?P<v>.+)\s"
        ).unwrap();
        static ref VALUES_RE: Regex = Regex::new(
            r"^(?P<fs>\S+)\s+\d+\s+(?P<u>\d+)\s+(?P<a>\d+)\s+\d+%.*$"
        ).unwrap();
    }

    let caps = CAP_LINE_RE.captures(df_str).ok_or_else(|| {
        RegexCaptureError::new(&CAP_LINE_RE, df_str.to_owned())
            .context(ErrorKind::RegexInitialHdfsDfCap)
    })?;

    let value_str = &caps["v"];

    let values = VALUES_RE.captures(value_str).ok_or_else(|| {
        RegexCaptureError::new(&VALUES_RE, value_str.to_owned())
            .context(ErrorKind::RegexHdfsDfValuesCap)
    })?;

    let filesystem = values["fs"].to_owned();

    let used = values["u"]
        .parse::<u64>()
        .context(ErrorKind::ParseHdfsDfUsedValue)?;

    let available = values["a"]
        .parse::<u64>()
        .context(ErrorKind::ParseHdfsDfSizeValue)?;

    Ok(StorageBuilder::default()
        .path(filesystem)
        .used(used)
        .capacity(used + available)
        .build())
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
        parse_df(df_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_df_parse_valid_1() {
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use%
            hdfs://localhost:8020 244529655808  3608576  161972236288    0%
            "
        );

        const USED: u64 = 3608576;
        const AVAILABLE: u64 = 161972236288;

        let storage = parse_df(TARGET);
        assert!(storage.is_ok());

        let storage = storage.unwrap();
        assert_eq!("hdfs://localhost:8020", storage.path());
        assert_eq!(USED + AVAILABLE, *storage.capacity());
        assert_eq!(USED, *storage.used());
        assert_eq!(AVAILABLE, *storage.remaining());
    }

    #[test]
    fn test_df_parse_invalid_1() {
        // no matching 'Filesystem'
        const TARGET: &str = indoc!(
            "
            FS                    Size          Used     Available       Use%
            hdfs://localhost:8020 244529655808  3608576  161972236288    0%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }

    #[test]
    fn test_df_parse_invalid_2() {
        // 'Use' instead of 'Use%'
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use
            hdfs://localhost:8020 244529655808  3608576  161972236288    0%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }

    #[test]
    fn test_df_parse_invalid_3() {
        // Size value is not numeric
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use%
            hdfs://localhost:8020 abc           3608576  161972236288    0%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }

    #[test]
    fn test_df_parse_invalid_4() {
        // Used value is not numeric
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use%
            hdfs://localhost:8020 244529655808  abc      161972236288    0%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }

    #[test]
    fn test_df_parse_invalid_5() {
        // Available value is not numeric
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use%
            hdfs://localhost:8020 244529655808  3608576  abc             0%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }

    #[test]
    fn test_df_parse_invalid_6() {
        // Use value is not numeric
        const TARGET: &str = indoc!(
            "
            Filesystem            Size          Used     Available       Use%
            hdfs://localhost:8020 244529655808  3608576  161972236288    abc%
            "
        );

        let storage = parse_df(TARGET);
        assert!(storage.is_err());
    }
}

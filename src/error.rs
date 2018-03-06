use failure::{Backtrace, Context, Fail};
use regex::Regex;
use std;
use std::fmt::{self, Display};
use std::path::PathBuf;

// suppress false positives from cargo-clippy
#[cfg_attr(feature = "cargo-clippy", allow(empty_line_after_outer_attr))]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Child output error")]
    ChildOutput,

    #[fail(display = "Default logger initialization error")]
    DefaultLoggerInit,

    #[fail(display = "Error piping password echo")]
    EchoPwPipe,

    #[fail(display = "File I/O error")]
    FileIo,

    #[fail(display = "Initial fluent post check error")]
    FluentInitCheck,

    #[fail(display = "Fluent post from tagged record error")]
    FluentPostTaggedRecord,

    #[fail(display = "Cannot find hdfs command from which")]
    HdfsNotAvailable,

    #[fail(display = "Error running hdfs dfs -df command")]
    HdfsDfCmd,

    #[fail(display = "kinit for username and keytab combi returns error")]
    KinitKeytab,

    #[fail(display = "Cannot find kinit command from which")]
    KinitNotAvailable,

    #[fail(display = "kinit for username and password combi returns error")]
    KinitPw,

    #[fail(display = "Lock file open error")]
    LockFileOpen,

    #[fail(display = "Lock file exclusive lock error")]
    LockFileExclusiveLock,

    #[fail(display = "Cannot parse hdfs dfs -df size value")]
    ParseHdfsDfSizeValue,

    #[fail(display = "Cannot parse hdfs dfs -df used value")]
    ParseHdfsDfUsedValue,

    #[fail(display = "Cannot capture values from hdfs dfs -df extraction")]
    RegexHdfsDfValuesCap,

    #[fail(display = "Cannot get initial hdfs dfs -df regex capture")]
    RegexInitialHdfsDfCap,

    #[fail(display = "Specialized logger initialization error")]
    SpecializedLoggerInit,

    #[fail(display = "Conversion from UTF8 stderr to string fail")]
    StderrUtf8Conversion,

    #[fail(display = "Stderr is empty")]
    StderrEmpty,

    #[fail(display = "Error reading from stderr pipe")]
    StderrRead,

    #[fail(display = "Error with message in stderr")]
    StderrValidMsg,

    #[fail(display = "Conversion from UTF8 stdout to string fail")]
    StdoutUtf8Conversion,

    #[fail(display = "TOML config parse error")]
    TomlConfigParse,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Debug, Fail)]
#[fail(display = "{{ code: {:?}, msg: {} }}", code, msg)]
pub struct CodeMsgError {
    pub code: Option<i32>,
    pub msg: String,
}

#[derive(Debug, Fail)]
#[fail(display = "{{ msg: {} }}", msg)]
pub struct MsgError {
    pub msg: String,
}

#[derive(Debug, Fail)]
#[fail(display = "{{ path: {:?}, inner: {} }}", path, inner)]
pub struct PathError<E>
where
    E: Fail,
{
    pub path: PathBuf,
    #[cause]
    pub inner: E,
}

#[derive(Debug, Fail, Getters)]
#[fail(display = "{{ pattern: {}, target: {} }}", pattern, target)]
pub struct RegexCaptureError {
    pattern: String,
    target: String,
}

impl RegexCaptureError {
    pub fn new<T>(pattern: &Regex, target: T) -> RegexCaptureError
    where
        T: Into<String>,
    {
        RegexCaptureError {
            pattern: pattern.as_str().to_owned(),
            target: target.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ inner: {}, cause: {:?}, backtrace: {:?} }}",
            self.inner,
            self.cause(),
            self.backtrace()
        )
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

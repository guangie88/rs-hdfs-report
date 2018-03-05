use failure::{Backtrace, Context, Fail};
use regex;
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

    #[fail(display = "Directory flag is unexpectedly empty")]
    DirFlagEmpty,

    #[fail(display = "Unable to create directories for copying to destination")]
    DirsCreate,

    #[fail(display = "Error piping password echo")]
    EchoPwPipe,

    #[fail(display = "File I/O error")]
    FileIo,

    #[fail(display = "Error invoking hdfs dfs -copyToLocal")]
    HdfsCopyToLocal,

    #[fail(display = "Error invoking hdfs dfs -ls")]
    HdfsDfsLs,

    #[fail(display = "Unable to find hdfs command from which")]
    HdfsNotAvailable,

    #[fail(display = "Error creating regex for hdfs matches")]
    HdfsRegexMatch,

    #[fail(display = "kinit for username and keytab combi returns error")]
    KinitKeytab,

    #[fail(display = "Unable to find kinit command from which")]
    KinitNotAvailable,

    #[fail(display = "kinit for username and password combi returns error")]
    KinitPw,

    #[fail(display = "Lock file open error")]
    LockFileOpen,

    #[fail(display = "Lock file exclusive lock error")]
    LockFileExclusiveLock,

    #[fail(display = "Unable to parse naive date time")]
    NaiveDateTimeParse,

    #[fail(display = "Unable to parse file size from regex capture")]
    RegexCapFileSizeParse,

    #[fail(display = "Unable to regex capture permissions")]
    RegexCapPerm,

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

    #[fail(display = "Unable to strip root '/' from path")]
    StripRootPath,

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

#[derive(Debug, Fail)]
#[fail(display = "{{ target: {}, inner: {} }}", target, inner)]
pub struct RegexError {
    pub target: String,
    pub inner: regex::Error,
}

#[derive(Debug, Fail)]
#[fail(display = "{{ perm: {} }}", perm)]
pub struct PermError {
    pub perm: String,
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

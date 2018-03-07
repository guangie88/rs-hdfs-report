use failure::{Fail, ResultExt};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::process::{Child, ChildStdout, Output};
use super::error::{CodeMsgError, Error, ErrorKind, MsgError, PathError, Result};

pub fn extract_child_stdout(child: Child) -> Result<ChildStdout> {
    let (stdout, stderr) = (child.stdout, child.stderr);

    let stdout = stdout.ok_or_else(|| {
        let msg_err = stderr
            .ok_or_else(|| -> Error { ErrorKind::StderrEmpty.into() })
            .and_then(|mut bytes| -> Result<Error> {
                let mut msg = String::new();

                bytes
                    .read_to_string(&mut msg)
                    .context(ErrorKind::StderrRead)?;

                Ok(MsgError::new(msg)
                    .context(ErrorKind::StderrValidMsg)
                    .into())
            });

        match msg_err {
            Ok(e) | Err(e) => e,
        }
    })?;

    Ok(stdout)
}

pub fn extract_output_stdout_str(output: Output) -> Result<String> {
    let output = if output.status.success() {
        String::from_utf8(output.stdout)
            .context(ErrorKind::StdoutUtf8Conversion)
    } else {
        let msg = String::from_utf8(output.stderr)
            .context(ErrorKind::StderrUtf8Conversion)?;

        Err(CodeMsgError::new(output.status.code(), msg))
            .context(ErrorKind::ChildOutput)
    }?;

    Ok(output)
}

pub fn read_from_file<P: AsRef<Path>>(p: P) -> Result<String> {
    let mut buf = String::new();
    let p = p.as_ref();

    let mut file = File::open(p)
        .map_err(|e| PathError::new(p.to_string_lossy().to_string(), e))
        .context(ErrorKind::FileIo)?;

    file.read_to_string(&mut buf)
        .context(ErrorKind::FileIo)?;
    Ok(buf)
}

pub fn lock_file<P: AsRef<Path>>(file_path: P) -> Result<File> {
    let file_path = file_path.as_ref();

    let flock = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|e| PathError::new(file_path, e))
        .context(ErrorKind::LockFileOpen)?;

    flock
        .try_lock_exclusive()
        .map_err(|e| PathError::new(file_path, e))
        .context(ErrorKind::LockFileExclusiveLock)?;

    Ok(flock)
}

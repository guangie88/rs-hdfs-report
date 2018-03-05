pub mod error;

use failure::{Fail, ResultExt};
use self::error::{CodeMsgError, Error, ErrorKind, MsgError, Result};
use std::io::Read;
use std::process::{Child, ChildStdout, Output};

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

                Ok(MsgError { msg }
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
        Err(CodeMsgError {
            code: output.status.code(),
            msg: String::from_utf8(output.stderr)
                .context(ErrorKind::StderrUtf8Conversion)?,
        }.context(ErrorKind::ChildOutput))
    }?;

    Ok(output)
}

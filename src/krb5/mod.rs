use failure::ResultExt;
use mega_coll::conf::krb5::Auth;
use mega_coll::error::{ErrorKind, Result};
use mega_coll::util::process::{extract_child_stdout, extract_output_stdout_str};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use which;

const ECHO: &str = "echo";
const KINIT: &str = "kinit";

#[derive(Debug)]
pub struct Krb5 {
    kinit: PathBuf,
}

impl Krb5 {
    pub fn new() -> Result<Krb5> {
        let kinit = which::which(KINIT).context(ErrorKind::KinitNotAvailable)?;
        Ok(Krb5::with_path(kinit))
    }

    pub fn with_path(kinit: PathBuf) -> Krb5 {
        Krb5 { kinit }
    }

    pub fn kinit(&self, name: &str, auth: &Auth) -> Result<String> {
        let kinit = match *auth {
            Auth::Password(ref pw) => {
                let echo = Command::new(ECHO)
                    .arg(pw.as_ref())
                    .stdout(Stdio::piped())
                    .spawn()
                    .context(ErrorKind::EchoPwPipe)?;

                let pw = extract_child_stdout(echo)?;

                Command::new(&self.kinit)
                    .arg(name)
                    .stdin(pw)
                    .output()
                    .context(ErrorKind::KinitPw)
            }

            Auth::Keytab(ref kt) => Command::new(&self.kinit)
                .args(&["-k", "-t", kt.as_ref(), name])
                .output()
                .context(ErrorKind::KinitKeytab),
        }?;

        extract_output_stdout_str(kinit)
    }
}

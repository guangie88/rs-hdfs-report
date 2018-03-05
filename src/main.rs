#![cfg_attr(feature = "cargo-clippy", deny(clippy))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
extern crate fs2;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_humantime;
extern crate simple_logger;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate toml;

mod hdfs;
mod krb5;
mod util;

use failure::ResultExt;
use fs2::FileExt;
use hdfs::Hdfs;
use krb5::Krb5;
use regex::Regex;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use util::error::{ErrorKind, PathError, RegexError, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-hdfs-to-local-conf",
            about = "Configuration for Rust HDFS-to-Local")]
struct ArgConf {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-hdfs-to-local.toml",
                help = "Configuration file path")]
    conf: String,
}

#[derive(Deserialize, Debug)]
struct Config<'a> {
    general: GeneralConfig,
    hdfs: HdfsConfig,
    kinit: KInitConfig<'a>,
}

#[derive(Deserialize, Debug)]
struct GeneralConfig {
    log_conf_path: Option<String>,
    lock_file: String,
    #[serde(with = "serde_humantime")]
    repeat_delay: Option<Duration>,
}

#[derive(Deserialize, Debug)]
struct HdfsConfig {
    path: String,
    matches: Vec<String>,
    copy_to: String,
}

#[derive(Deserialize, Debug)]
struct KInitConfig<'a> {
    login: String,
    auth: krb5::Auth<'a>,
}

fn strip_root(path: &Path) -> Result<&Path> {
    let path = if path.has_root() {
        path.strip_prefix("/")
            .map_err(|e| PathError {
                path: path.to_owned(),
                inner: e,
            })
            .context(ErrorKind::StripRootPath)?
    } else {
        path
    };

    Ok(path)
}

pub fn read_from_file<P: AsRef<Path>>(p: P) -> Result<String> {
    let mut buf = String::new();
    let mut file = File::open(p.as_ref()).context(ErrorKind::FileIo)?;
    file.read_to_string(&mut buf)
        .context(ErrorKind::FileIo)?;
    Ok(buf)
}

fn hdfs_recurse(
    hdfs: &Hdfs,
    copy_to_root: &str,
    path: &str,
    re_matches: &[Regex],
) -> Result<()> {
    let entries = hdfs.ls(path)?;

    for entry in &entries {
        if entry.is_dir {
            hdfs_recurse(hdfs, copy_to_root, &entry.path, re_matches)?;
        } else {
            // only apply matches on files
            let is_match = re_matches
                .iter()
                .any(|re| re.is_match(&entry.path));

            if is_match {
                let copy_from = &entry.path;
                let copy_from_stripped = strip_root(Path::new(copy_from))?;
                let copy_from_dir_stripped = copy_from_stripped.parent();

                if let Some(copy_from_dir_stripped) = copy_from_dir_stripped {
                    let copy_to_dir =
                        Path::new(copy_to_root).join(copy_from_dir_stripped);

                    fs::create_dir_all(copy_to_dir.clone())
                        .map_err(|e| PathError {
                            path: copy_to_dir,
                            inner: e,
                        })
                        .context(ErrorKind::DirsCreate)?;
                }

                let copy_to = Path::new(copy_to_root).join(copy_from_stripped);

                if copy_to.exists() {
                    debug!(
                        "NOT COPYING {:?}, {:?} EXISTS",
                        copy_from, copy_to
                    );
                } else {
                    hdfs.copy_to_local(copy_from, &copy_to.to_string_lossy())?;
                    debug!("COPY {:?} TO {:?}", copy_from, copy_to);
                }
            }
        }
    }

    Ok(())
}

fn run_impl(conf: &Config) -> Result<()> {
    let re_matches = conf.hdfs
        .matches
        .iter()
        .map(|target| -> Result<Regex> {
            let re = Regex::new(target)
                .map_err(|e| RegexError {
                    target: target.clone(),
                    inner: e,
                })
                .context(ErrorKind::HdfsRegexMatch)?;

            Ok(re)
        })
        .collect::<Result<Vec<Regex>>>()?;

    let krb5 = Krb5::new()?;
    let hdfs = Hdfs::new()?;

    krb5.kinit(&conf.kinit.login, &conf.kinit.auth)?;
    debug!("Kerberos kinit is successful");
    hdfs_recurse(
        &hdfs,
        &conf.hdfs.copy_to,
        &conf.hdfs.path,
        &re_matches,
    )?;

    Ok(())
}

fn run(conf: &Config) -> Result<()> {
    // to check if the process is already running as another PID
    let _flock = lock_file(&conf.general.lock_file)?;

    match conf.general.repeat_delay {
        Some(repeat_delay) => loop {
            print_run_status(&run_impl(conf));
            thread::sleep(repeat_delay)
        },
        None => run_impl(conf),
    }
}

fn init<'a>() -> Result<Config<'a>> {
    let arg_conf = ArgConf::from_args();

    let conf: Config = toml::from_str(&read_from_file(&arg_conf.conf)?)
        .context(ErrorKind::TomlConfigParse)?;

    match conf.general.log_conf_path {
        Some(ref log_conf_path) => {
            log4rs::init_file(log_conf_path, Default::default())
                .context(ErrorKind::SpecializedLoggerInit)?
        }
        None => simple_logger::init().context(ErrorKind::DefaultLoggerInit)?,
    }

    Ok(conf)
}

fn lock_file<P: AsRef<Path>>(file_path: P) -> Result<File> {
    let file_path = file_path.as_ref();

    let flock = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .map_err(|e| PathError {
            path: file_path.to_owned(),
            inner: e,
        })
        .context(ErrorKind::LockFileOpen)?;

    flock
        .try_lock_exclusive()
        .map_err(|e| PathError {
            path: file_path.to_owned(),
            inner: e,
        })
        .context(ErrorKind::LockFileExclusiveLock)?;

    Ok(flock)
}

fn print_run_status(res: &Result<()>) {
    match *res {
        Ok(_) => info!("Session completed!"),
        Err(ref e) => {
            error!("{}", e);
        }
    }
}

fn main() {
    let conf_res = init();

    if let Err(ref e) = conf_res {
        eprintln!("{}", e);
    }

    let res = conf_res.and_then(|conf| {
        info!("Program started!");
        debug!("```\n{:#?}```", conf);
        run(&conf)
    });

    print_run_status(&res);

    if res.is_err() {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_root_one() {
        let r = strip_root(Path::new("/foo"));
        assert!(r.is_ok());

        let p = r.unwrap();
        assert!(p == Path::new("foo"));
    }

    #[test]
    fn strip_root_two() {
        let r = strip_root(Path::new("//a/b"));
        assert!(r.is_ok());

        let p = r.unwrap();
        assert!(p == Path::new("a/b"));
    }

    #[test]
    fn strip_root_multi() {
        let r = strip_root(Path::new("////////a/b/c"));
        assert!(r.is_ok());

        let p = r.unwrap();
        assert!(p == Path::new("a/b/c"));
    }
}

#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

#[macro_use]
extern crate derive_getters;
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
use hdfs::Hdfs;
use krb5::Krb5;
use std::process;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use util::error::{ErrorKind, Result};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-hdfs-report-conf",
            about = "Configuration for Rust hdfs-report")]
struct ArgConf {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-hdfs-report.toml",
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

fn run_impl(conf: &Config) -> Result<()> {
    let krb5 = Krb5::new()?;
    let hdfs = Hdfs::new()?;

    krb5.kinit(&conf.kinit.login, &conf.kinit.auth)?;
    debug!("Kerberos kinit is successful");
    let storage = hdfs.df("/")?;

    Ok(())
}

fn run(conf: &Config) -> Result<()> {
    // to check if the process is already running as another PID
    let _flock = util::lock_file(&conf.general.lock_file)?;

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

    let conf: Config = toml::from_str(&util::read_from_file(&arg_conf.conf)?)
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
}

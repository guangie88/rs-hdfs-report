#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

extern crate chrono;
extern crate failure;
extern crate fruently;
extern crate fs2;
extern crate json_collection;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate mega_coll;
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
extern crate which;

#[cfg(test)]
#[macro_use]
extern crate indoc;

mod conf;
mod hdfs;
mod krb5;

use conf::{ArgConf, Config};
use failure::ResultExt;
use fruently::forwardable::JsonForwardable;
use hdfs::Hdfs;
use krb5::Krb5;
use mega_coll::error::{ErrorKind, Result};
use mega_coll::error::custom::PathError;
use mega_coll::util::app::{create_and_check_fluent, print_run_status,
                           read_config_file};
use mega_coll::util::fs::lock_file;
use std::process;
use std::thread;
use structopt::StructOpt;

fn run_impl(conf: &Config) -> Result<()> {
    let fluent = create_and_check_fluent(
        &conf.fluentd,
        "rs-hdfs-report-log-initialization",
    )?;

    let krb5 = Krb5::new()?;
    let hdfs = Hdfs::new()?;

    krb5.kinit(&conf.kinit.login, &conf.kinit.auth)?;
    debug!("Kerberos kinit is successful");

    let storage = hdfs.df("/")?;

    fluent
        .clone()
        .post(&storage)
        .context(ErrorKind::FluentPostTaggedRecord)?;

    Ok(())
}

fn run(conf: &Config) -> Result<()> {
    // to check if the process is already running as another PID
    let _flock = lock_file(&conf.general.lock_file)?;

    match conf.general.repeat_delay {
        Some(repeat_delay) => loop {
            print_run_status(&run_impl(conf), "Session completed!");
            thread::sleep(repeat_delay)
        },
        None => run_impl(conf),
    }
}

fn init<'a>() -> Result<Config<'a>> {
    let arg_conf = ArgConf::from_args();
    let conf: Config = read_config_file(&arg_conf.conf)?;

    match conf.general.log_conf_path {
        Some(ref log_conf_path) => {
            log4rs::init_file(log_conf_path, Default::default())
                .map_err(|e| PathError::new(log_conf_path, e))
                .context(ErrorKind::SpecializedLoggerInit)?
        }
        None => simple_logger::init().context(ErrorKind::DefaultLoggerInit)?,
    }

    Ok(conf)
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

    print_run_status(&res, "Program completed!");

    if res.is_err() {
        process::exit(1);
    }
}

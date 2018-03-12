#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

extern crate failure;
extern crate fruently;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate mega_coll;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate which;

#[cfg(test)]
#[macro_use]
extern crate indoc;

mod conf;
mod hdfs;
mod krb5;

use conf::{ArgConfig, Config};
use failure::ResultExt;
use fruently::forwardable::JsonForwardable;
use hdfs::Hdfs;
use krb5::Krb5;
use mega_coll::error::{ErrorKind, Result};
use mega_coll::util::app::{create_and_check_fluent, init_config,
                           print_run_status};
use mega_coll::util::fs::lock_file;
use std::process;
use std::thread;

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

fn main() {
    let conf_res = init_config::<ArgConfig, Config, ErrorKind>();

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

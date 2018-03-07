#![cfg_attr(feature = "cargo-clippy", deny(warnings))]

#[macro_use]
extern crate failure;
extern crate fruently;
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

#[cfg(test)]
#[macro_use]
extern crate indoc;

mod conf;
mod error;
mod hdfs;
mod krb5;
mod util;

use conf::{ArgConf, Config};
use error::{ErrorKind, PathError, Result};
use failure::ResultExt;
use fruently::fluent::Fluent;
use fruently::forwardable::JsonForwardable;
use fruently::retry_conf::RetryConf;
use hdfs::Hdfs;
use krb5::Krb5;
use std::path::Path;
use std::process;
use std::thread;
use structopt::StructOpt;

fn create_and_check_fluent<'a>(
    conf: &'a Config,
) -> Result<Fluent<'a, &'a String>> {
    let fluent_conf = RetryConf::new()
        .max(conf.fluentd.try_count)
        .multiplier(conf.fluentd.multiplier);

    let fluent_conf = match conf.fluentd.store_file_path {
        Some(ref store_file_path) => {
            fluent_conf.store_file(Path::new(store_file_path).to_owned())
        }
        None => fluent_conf,
    };

    let fluent = Fluent::new_with_conf(
        &conf.fluentd.address,
        conf.fluentd.tag.as_str(),
        fluent_conf,
    );

    fluent
        .clone()
        .post("rs-hdfs-report-log-initialization")
        .context(ErrorKind::FluentInitCheck)?;

    Ok(fluent)
}

fn read_config_file<'a, P>(conf_path: P) -> Result<Config<'a>>
where
    P: AsRef<Path>,
{
    let conf_path = conf_path.as_ref();

    let config: Config = toml::from_str(&util::read_from_file(conf_path)?)
        .map_err(|e| PathError::new(conf_path, e))
        .context(ErrorKind::TomlConfigParse)?;

    Ok(config)
}

fn run_impl(conf: &Config) -> Result<()> {
    let fluent = create_and_check_fluent(conf)?;

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
    let conf: Config = read_config_file(&arg_conf.conf)?;

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

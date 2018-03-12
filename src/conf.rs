use mega_coll::conf::{app, fluentd, hdfs, krb5};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-hdfs-report-conf",
            about = "Configuration for Rust hdfs-report")]
pub struct ArgConfig {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-hdfs-report.toml",
                help = "Configuration file path")]
    pub conf: String,
}

impl app::ArgConf for ArgConfig {
    fn conf(&self) -> &str {
        &self.conf
    }
}

#[derive(Deserialize, Debug)]
pub struct Config<'a> {
    pub general: app::Config,
    pub fluentd: fluentd::Config,
    pub hdfs: hdfs::Config,
    pub kinit: krb5::Config<'a>,
}

impl<'a> app::Conf for Config<'a> {
    fn general(&self) -> &app::Config {
        &self.general
    }
}

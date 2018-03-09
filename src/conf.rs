use mega_coll::conf::{app, fluentd, hdfs, krb5};

#[derive(StructOpt, Debug)]
#[structopt(name = "rs-hdfs-report-conf",
            about = "Configuration for Rust hdfs-report")]
pub struct ArgConf {
    #[structopt(short = "c", long = "conf",
                default_value = "config/rs-hdfs-report.toml",
                help = "Configuration file path")]
    pub conf: String,
}

#[derive(Deserialize, Debug)]
pub struct Config<'a> {
    pub general: app::Config,
    pub fluentd: fluentd::Config,
    pub hdfs: hdfs::Config,
    pub kinit: krb5::Config<'a>,
}

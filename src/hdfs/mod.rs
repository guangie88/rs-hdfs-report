extern crate chrono;
extern crate which;

mod bits;

use failure::ResultExt;
use regex::Regex;
use self::chrono::NaiveDateTime;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str;

use self::bits::SubPerm;
use super::util::extract_output_stdout_str;
use super::util::error::{ErrorKind, PermError, Result};

const HDFS: &str = "hdfs";
const DFS: &str = "dfs";

#[derive(Debug)]
pub struct Hdfs {
    p: PathBuf,
}

#[derive(Debug)]
pub struct Perm {
    user: SubPerm,
    group: SubPerm,
    others: SubPerm,
}

#[derive(Debug)]
pub struct Entry {
    pub is_dir: bool,
    pub perm: Perm,
    pub replication: u64,
    pub user: String,
    pub group: String,
    pub filesize: u64,
    pub datetime: NaiveDateTime,
    pub path: String,
}

impl Hdfs {
    pub fn new() -> Result<Hdfs> {
        let p = which::which(HDFS).context(ErrorKind::HdfsNotAvailable)?;
        Ok(Hdfs::with_path(p))
    }

    pub fn with_path(p: PathBuf) -> Hdfs {
        Hdfs { p }
    }

    pub fn ls(&self, path: &str) -> Result<Vec<Entry>> {
        let ls = Command::new(&self.p)
            .args(&[DFS, "-ls", path])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(ErrorKind::HdfsDfsLs)?;

        let ls_str = extract_output_stdout_str(ls)?;

        lazy_static! {
            static ref CLEAN_RE: Regex =
                Regex::new(r"Found \d+ items\n")
                .unwrap();

            static ref CAP_RE: Regex =
                Regex::new(r"(?P<pe>\S+)\s+(?P<rep>\S+)\s+(?P<u>\S+)\s+(?P<g>\S+)\s+(?P<fs>\d+)\s+(?P<d>\S+)\s+(?P<t>\S+)\s+(?P<pa>.+)\n")
                .unwrap();
        }

        let mod_ls = CLEAN_RE.replace(&ls_str, "");
        let caps = CAP_RE.captures_iter(&mod_ls);

        let entries = caps.map(|c| -> Result<Entry> {
            let mut perm = c["pe"].chars();
            let dir_flag = perm.next()
                .ok_or_else(|| ErrorKind::DirFlagEmpty)?;

            let is_dir = dir_flag == 'd';
            let perm: String = perm.collect();

            let datetime = NaiveDateTime::parse_from_str(
                &format!("{} {}", &c["d"], &c["t"]),
                "%Y-%m-%d %H:%M",
            ).context(ErrorKind::NaiveDateTimeParse)?;

            let san_path = sanitize_path(&c["pa"]);

            Ok(Entry {
                is_dir,
                perm: parse_perm(&perm)?,
                replication: c["rep"].parse().unwrap_or(0),
                user: c["u"].to_owned(),
                group: c["g"].to_owned(),
                filesize: c["fs"]
                    .parse::<u64>()
                    .context(ErrorKind::RegexCapFileSizeParse)?,
                datetime,
                path: san_path,
            })
        });

        Ok(entries.collect::<Result<Vec<Entry>>>()?)
    }

    pub fn copy_to_local(&self, src: &str, dst: &str) -> Result<String> {
        let copy_to_local = Command::new(&self.p)
            .args(&[DFS, "-copyToLocal", src, dst])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(ErrorKind::HdfsCopyToLocal)?;

        extract_output_stdout_str(copy_to_local)
    }
}

fn sanitize_path(path: &str) -> String {
    lazy_static! {
        static ref HDFS_RE: Regex =
            Regex::new(r"^\w+://.+?(?::\d+)?(?P<p>/.*)$").unwrap();
    }

    let hdfs_path_cap = HDFS_RE.captures(path);

    if let Some(hdfs_path_cap) = hdfs_path_cap {
        hdfs_path_cap["p"].to_owned()
    } else {
        path.to_owned()
    }
}

fn parse_sub_perm(r: &str, w: &str, x: &str) -> SubPerm {
    let rb = if r == "r" {
        SubPerm::READ
    } else {
        SubPerm::NIL
    };

    let wb = if w == "w" {
        SubPerm::WRITE
    } else {
        SubPerm::NIL
    };

    let xb = if x == "x" {
        SubPerm::EXEC
    } else {
        SubPerm::NIL
    };

    rb | wb | xb
}

fn parse_perm(perm: &str) -> Result<Perm> {
    lazy_static! {
        static ref PERM_RE: Regex =
            Regex::new(r"^(?P<ur>r|-)(?P<uw>w|-)(?P<ux>x|-)(?P<gr>r|-)(?P<gw>w|-)(?P<gx>x|-)(?P<or>r|-)(?P<ow>w|-)(?P<ox>x|-)$")
            .unwrap();
    }

    let caps = PERM_RE
        .captures(perm)
        .ok_or_else(|| PermError {
            perm: perm.to_owned(),
        })
        .context(ErrorKind::RegexCapPerm)?;

    let user = parse_sub_perm(&caps["ur"], &caps["uw"], &caps["ux"]);
    let group = parse_sub_perm(&caps["gr"], &caps["gw"], &caps["gx"]);
    let others = parse_sub_perm(&caps["or"], &caps["ow"], &caps["ox"]);

    Ok(Perm {
        user,
        group,
        others,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path_full() {
        assert!(sanitize_path("hdfs://localhost:8020/") == "/");
    }

    #[test]
    fn test_sanitize_path_full_extended() {
        assert!(sanitize_path("hdfs://localhost:8020/a/b/c") == "/a/b/c");
    }

    #[test]
    fn test_sanitize_path_full_extended_space() {
        assert!(
            sanitize_path(r"hdfs://localhost:8020/\ /\ /foo") == r"/\ /\ /foo"
        );
    }

    #[test]
    fn test_sanitize_path_without_port() {
        assert!(sanitize_path("hdfs://localhost/") == "/");
    }

    #[test]
    fn test_sanitize_path_without_port_extended() {
        assert!(sanitize_path("hdfs://localhost/foo/bar") == "/foo/bar");
    }

    #[test]
    fn test_sanitize_path_without_port_extended_space() {
        assert!(sanitize_path(r"hdfs://localhost/\  /foo") == r"/\  /foo");
    }

    #[test]
    fn test_sanitize_path_root() {
        assert!(sanitize_path("/") == "/");
    }

    #[test]
    fn test_sanitize_path_extended() {
        assert!(sanitize_path("/a/b/c") == "/a/b/c");
    }

    #[test]
    fn test_sanitize_path_non_root() {
        assert!(sanitize_path("abc") == "abc");
    }

    #[test]
    fn test_sanitize_path_empty() {
        assert!(sanitize_path("") == "");
    }

    #[test]
    fn test_parse_sub_perm_all() {
        let p = parse_sub_perm("r", "w", "x");
        assert!(p == SubPerm::ALL);
    }

    #[test]
    fn test_parse_sub_perm_none() {
        let p = parse_sub_perm("-", "-", "-");
        assert!(p == SubPerm::NIL);
    }

    #[test]
    fn test_parse_sub_perm_none_alt1() {
        // only one char is allowable for sub-parsing
        let p = parse_sub_perm(" r", " w", "x ");
        assert!(p == SubPerm::NIL);
    }

    #[test]
    fn test_parse_sub_perm_none_alt2() {
        let p = parse_sub_perm("R", "W", "X");
        assert!(p == SubPerm::NIL);
    }

    #[test]
    fn test_parse_sub_perm_read_only() {
        let p = parse_sub_perm("r", "-", "-");
        assert!(p == SubPerm::READ);
    }

    #[test]
    fn test_parse_sub_perm_write_only() {
        let p = parse_sub_perm("-", "w", "-");
        assert!(p == SubPerm::WRITE);
    }

    #[test]
    fn test_parse_sub_perm_exec_only() {
        let p = parse_sub_perm("-", "-", "x");
        assert!(p == SubPerm::EXEC);
    }

    #[test]
    fn test_parse_sub_perm_read_write() {
        let p = parse_sub_perm("r", "w", "-");
        assert!(p == SubPerm::READ | SubPerm::WRITE);
    }

    #[test]
    fn test_parse_sub_perm_read_exec() {
        let p = parse_sub_perm("r", "-", "x");
        assert!(p == SubPerm::READ | SubPerm::EXEC);
    }

    #[test]
    fn test_parse_sub_perm_write_exec() {
        let p = parse_sub_perm("-", "w", "x");
        assert!(p == SubPerm::WRITE | SubPerm::EXEC);
    }

    #[test]
    fn test_parse_perm_all() {
        let r = parse_perm("rwxrwxrwx");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::ALL);
        assert!(perms.group == SubPerm::ALL);
        assert!(perms.others == SubPerm::ALL);
    }

    #[test]
    fn test_parse_perm_none() {
        let r = parse_perm("---------");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::NIL);
        assert!(perms.group == SubPerm::NIL);
        assert!(perms.others == SubPerm::NIL);
    }

    #[test]
    fn test_parse_perm_user() {
        let r = parse_perm("rwx------");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::ALL);
        assert!(perms.group == SubPerm::NIL);
        assert!(perms.others == SubPerm::NIL);
    }

    #[test]
    fn test_parse_perm_group() {
        let r = parse_perm("---rwx---");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::NIL);
        assert!(perms.group == SubPerm::ALL);
        assert!(perms.others == SubPerm::NIL);
    }

    #[test]
    fn test_parse_perm_others() {
        let r = parse_perm("------rwx");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::NIL);
        assert!(perms.group == SubPerm::NIL);
        assert!(perms.others == SubPerm::ALL);
    }

    #[test]
    fn test_parse_perm_complex1() {
        let r = parse_perm("r-xrw---x");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::READ | SubPerm::EXEC);
        assert!(perms.group == SubPerm::READ | SubPerm::WRITE);
        assert!(perms.others == SubPerm::EXEC);
    }

    #[test]
    fn test_parse_perm_complex2() {
        let r = parse_perm("--x---rwx");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::EXEC);
        assert!(perms.group == SubPerm::NIL);
        assert!(perms.others == SubPerm::ALL);
    }

    #[test]
    fn test_parse_perm_complex3() {
        let r = parse_perm("-wxrw-r-x");
        assert!(r.is_ok());

        let perms = r.unwrap();
        assert!(perms.user == SubPerm::WRITE | SubPerm::EXEC);
        assert!(perms.group == SubPerm::READ | SubPerm::WRITE);
        assert!(perms.others == SubPerm::READ | SubPerm::EXEC);
    }

    #[test]
    fn test_parse_perm_excess_fail() {
        assert!(parse_perm(" rwxrwxrwx ").is_err());
    }
}

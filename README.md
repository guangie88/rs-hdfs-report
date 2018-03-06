# `rs-hdfs-report`

Rust project to perform `hdfs dfs -df` command call to the Kerberos protected
HDFS server to. Relies heavily on the built Docker environment to work since the
project assumes `hdfs` and `kinit` executables to be present.

A corresponding Kerberos protected HDFS server is also available in `server/`
for demonstration purposes.

## Requirements to run

* `docker`
* `docker-compose`

## Commands to run for server

Read the instructions in `server/README.md`.

## Commands to run for client

```bash
docker-compose up --build app
```

The above command will build the Docker image with the statically compiled
`rs-hdfs-report` executable, together with `config/rs-hdfs-report.toml` and
`config/rs-hfds-to-local-log.yml`.

The above command is only useful if there is a corresponding HDFS server and a
Fluentd server.

## Storage JSON format

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct Storage {
    path: String,
    capacity: u64,
    used: u64,
    remaining: u64,
    used_prop: f64,
    remaining_prop: f64,
    datetime: DateTime<Local>,
}
```

can for example become:

```json
{
  "path": "/",
  "capacity": 1000,
  "used": 250,
  "remaining": 750,
  "used_prop": 0.25,
  "remaining_prop": 0.75,
  "datetime": "2017-01-20T13:08:35.000000000+08:00"
}
```

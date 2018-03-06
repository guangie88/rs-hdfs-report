# Test services for `rs-hdfs-report`

The `docker-compose.yml` contains all the services to run in order to observe
the log output of the storage diagnostics from `rs-hdfs-report`. The services
are meant to run on `localhost`.

## What to run

Run the following:

```sh
docker-compose up --build -d
```

to start all the services. The services are as follow:

* `fluentd`
  * Log interface for `rs-hdfs-report`
* `elasticsearch`
  * Log backend for storage
* `grafana`
  * Log visualization for observing storage diagnostics
* `grafana-import`
  * Add `elasticsearch` into `grafana`, and import dashboard for log
    visualization.
  * Non-persistent, spins up and immediately spins down.
* `hdfs-krb5`
  * HDFS + Kerberos server to performing the `hdfs dfs` commands on.
  * The keytab file in `rs-hdfs-report.toml` matches the one used in this
    server.

## How to observe

Open <http://localhost:3000> and log in with:

* Username: `admin`
* Password: `admin`

Open up `Diagnostics` dashboard. If `rs-hdfs-report` executable is already
running, you should see new log points on every 10 second refresh.

If `rs-hdfs-report` is not running yet, open a new console, navigate to this
repository root directory and run:

```sh
cargo run --release
```

The above assumes that the Rust environment has already been set up.

## How to spin down

Run the following:

```sh
docker-compose down -v
```

This stops all services and removes all local volumes, i.e. previous logs and
imports are removed, making the session ephemeral.

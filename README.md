# `rs-hdfs-to-local`

Rust project to perform HDFS file copy from Kerberos protected HDFS server to
local storage, via regex matches. Relies heavily on the built Docker environment
to work since the project assumes `hdfs` and `kinit` executables to be present.

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
`rs-hdfs-to-local` executable, together with `config/rs-hdfs-to-local.toml` and
`config/rs-hfds-to-local-log.yml`.

The above command is only useful if there is a corresponding server to respond
to.

The current configuration dictates the executable to run forever, and the
following files from the HDFS server will be copied into the directory
`dst/filesystem-test-fixture/links/`:

* `file0`
* `file1`

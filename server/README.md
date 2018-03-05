# Test Server for `rs-hdfs-to-local`

The test server uses Docker image `nlesc/xenon-hdfs-kerberos:latest` and exposes
the relevant ports.

To use it on `localhost`, run the following:
`docker-compose up -d --build app`.

The service takes awhile before starting up both the Kerberos and Hadoop
services, so it is advisable to wait for about 30 secs before testing out the
service.

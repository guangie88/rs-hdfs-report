# compilation
FROM clux/muslrust:nightly-2018-02-26 as builder

WORKDIR /app
COPY ./src/ src/
COPY ./Cargo.lock ./Cargo.toml ./

RUN set -x \
    && printf "nameserver 8.8.8.8\nnamespace 8.8.4.4" > /etc/resolv.conf \
    && cargo fetch --locked -v \
    && cargo build --target=x86_64-unknown-linux-musl --release --frozen -v \
    && mv target/x86_64-unknown-linux-musl/release/rs-hdfs-report ./ \
    && rm -rf Cargo.lock Cargo.toml src/ target/

# runtime
FROM guangie88/hdfs-client-krb5-xenon:latest

WORKDIR /app
COPY --from=builder /app/rs-hdfs-report ./
COPY ./run.sh ./
COPY ./config/ config/

ENTRYPOINT [ "./run.sh" ]

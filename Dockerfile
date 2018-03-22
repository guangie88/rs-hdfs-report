# compilation
FROM clux/muslrust:nightly-2018-02-26 as builder
WORKDIR /app

COPY ./Cargo.lock ./Cargo.toml ./
RUN set -x && cargo fetch --locked -v
COPY ./src/ src/

RUN cargo build --target=x86_64-unknown-linux-musl --release --frozen -v \
    && mv target/x86_64-unknown-linux-musl/release/rs-hdfs-report ./ \
    && rm -rf Cargo.lock Cargo.toml src/ target/

# runtime
FROM guangie88/hdfs-client-krb5-xenon:latest
WORKDIR /app

COPY --from=builder /app/rs-hdfs-report ./
COPY ./run.sh ./
COPY ./config/ config/

ENTRYPOINT [ "./run.sh" ]

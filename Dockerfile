ARG BUILDER_IMG=ekidd/rust-musl-builder:latest

FROM ${BUILDER_IMG} AS builder
LABEL stage=builder
WORKDIR /gyro
COPY . .
RUN sudo chown -R rust:rust .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates

COPY --from=builder \
  /gyro/target/x86_64-unknown-linux-musl/release/gyro \
  /usr/local/bin/
CMD /usr/local/bin/gyro

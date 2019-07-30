ARG BUILDER_IMG=ekidd/rust-musl-builder:latest

FROM ${BUILDER_IMG} AS builder
LABEL stage=builder
ADD . ./gyro
RUN sudo chown -R rust:rust ./gyro
RUN cd ./gyro && cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates

COPY --from=builder \
  /home/rust/src/gyro/target/x86_64-unknown-linux-musl/release/gyro \
  /usr/local/bin/
CMD /usr/local/bin/gyro

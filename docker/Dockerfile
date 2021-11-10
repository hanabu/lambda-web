#!/usr/bin/env buildah bud --layers=true -t lambda_builder

FROM amazonlinux:2

# Setup build environment
RUN mkdir -p /build/src && \
    yum update -y && \
# Add required packages
    yum install -y awscli gcc openssl-devel tree zip && \
# Install rust with rustup
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

# Build environment setting
WORKDIR /build
ENV PATH=/root/.cargo/bin:/usr/sbin:/usr/bin:/sbin:/bin

CMD \
  cargo build --release --target-dir target_lambda && \
  size target_lambda/release/bootstrap && \
  ldd  target_lambda/release/bootstrap && \
  zip -9 -j target_lambda/deploy.zip target_lambda/release/bootstrap

# Run container and build
#
# podman run -it --rm -v ~/.cargo/registry:/root/.cargo/registry:z -v .:/build:z lambda_builder
#


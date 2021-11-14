# Build and deploy Rust binary to Lambda

## TL;DR

- I reccomend ZIP deploy to `provided.al2` than container deploy, because of faster cold start time.
- To avoid shared library version mismatch, run `cargo build` on Amazon Linux 2 environment.
- Using CodeBuild may be the simplest way. You can also build [Arm64 binary](https://aws.amazon.com/blogs/news/aws-lambda-functions-powered-by-aws-graviton2-processor-run-your-functions-on-arm-and-get-up-to-34-better-price-performance/) if you prefer it.
- This repository contains [sample buildspec.yml](./buildspec.yml) for this purpose.

## Deploy Rust binary to AWS Lambda

At the time of writing (Nov, 2021), Amazon AWS supports two ways to deploy Rust binary to AWS Lambda.

- ZIP deploy to custom runtime `provided.al2`
- Container image deploy

Both work well, but cold start time is differ.
In my measurement, cold start time of ZIP deploy is around 250ms, while container deploy is around 600ms.
(Measurement is end-to-end, it's time includes network latency, TLS handshake, API gateway processing time, etc.)

## How to build binary for `provided.al2` runtime

If you have a Linux system, you can run `cargo build` to get binary. But sometimes you get error on runtime like

```txt
/lib64/libc.so.6: version `GLIBC_2.18` not found
```

This error is occured becasuse your system has newer version of libc than Amazon Linux 2 runtime.

So, I recommend to run `cargo build` on Amazon Linux 2 environment.
There are a few way to do it, I think [AWS CodeBuild](https://aws.amazon.com/codebuild/) is the simplest way.
Since CodeBuild provides Amazon Linux 2 as the standard build environment, you only need to place `buildspec.yml` file in the repository.

## Using AWS CodeBuild

- Write `buildspec.yml`
- Prepare S3 bucket where built binary will be placed.
- Create CodeBuild project
- Deploy to Lambda

First, place following `buildspec.yml` in your repository root directory. Make sure, `YOUR_PROJECT_NAME` is for your project.

```yml
version: 0.2

env:
  variables:
     PATH: "/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/codebuild/user/bin"
phases:
  install:
    commands:
      # Install rust toolchanin
      - curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  build:
    commands:
      # build Rust binary
      - cargo build --release
      # binary name must be "bootstrap" for Lambda custom runtime
      - mv target/release/YOUR_PROJECT_NAME bootstrap
      # strip & check size, dependencies
      - strip --strip-all bootstrap
      - size bootstrap
      - ldd bootstrap
artifacts:
  files:
    - bootstrap
    # - add other resources such as CSS, Javascript assets, etc.
  #discard-paths: yes
```

Second, prepare S3 bucket in same region with Lambda and CodeBuild. If you have already some bucket, you can use it. Or make new one.

Third, create new CodeBuild project.

- `Source` section: \
  Select source code repository of your project.
  CodeBuild can fetch CodeCommit, GitHub, Bitbucket repositories directly, or you should pack all source codes into ZIP and upload it to S3.
- `Environment` section:
  - Select `Managed Image`
  - Operating system - `Amazon Linux 2`
  - Runtime - `Standard`
  - Image - `aws/codebuild/amazonlinux2-aarch64-standard:2.0` or `aws/codebuild/amazonlinux2-x86_64-standard:3.0` (as of writing; Newer image will be come)
  - Environment type - `Linux`
  - Privileged - `disable`
  - Service role - If you unsure, select `New service role` and name it. If you use existing role, the role needs S3:PutObject permision to the bucket mentioned above.
- `Buildspec` section: \
  `Use a build file` and keep `Buildspec name` as blank, if you place buildspec.yml on your repository root.
  In other case, specify where your buildspec.yml is.
- `Batch configuration` section: \
  Disable `Define batch configuration`
- `Artifacts` section:
  - Set type as `Amazon S3` and specify `Bucket name` as the S3 bucket mentioned above.
  - `Name` is ZIP file name like lambda_deploy.zip
  - `Path` is directory of ZIP file. ZIP file will be placed at `s3://<Bucket name>/<Path>/<Name>`
  - `Namespace` - None
  - `Artifacts packaging` - Select `ZIP`

Congratulations! You can now `Start build` and get ZIPed Rust binary in S3 bucket.

Finally you can deploy binary to Lambda. Since ZIP is already on S3 bucket, `Upload from` - `Amazon S3 location` in the Lambda console.

## Another way to build - run Amazon Linux 2 Docker container on your machine

If you are familier with Docker, you can build Rust binary with Amazon Linux 2 container image.

First, build Amazon Linux 2 container image including Rust compiler toolchain with following Dockerfile.

```Dockerfile
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
# Default build command
CMD \
  cargo build --release --target-dir target_al2 && \
  mv target_al2/release/YOUR_PROJECT_NAME bootstrap && \
  strip --strip-all bootstrap && \
  size bootstrap && \
  ldd  bootstrap && \
  zip -9 -j deploy.zip bootstrap
```

Then, run container with mounting source code directory.

```console
$ cd /your/cargo-project/top
$ ls
Cargo.lock  Cargo.toml  src
$ docker run -it --rm \
  -v ~/.cargo/registry:/root/.cargo/registry:z \
  -v .:/build:z \
  BUILD_CONTAINER_NAME
...
$ ls
Cargo.lock  Cargo.toml  bootstrap  deploy.zip  src  target_al2
```

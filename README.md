[![Build Status](https://github.com/joaofl/quick-serve/actions/workflows/rust.yml/badge.svg)](https://github.com/joaofl/quick-serve/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/quick-serve.svg)](https://crates.io/crates/quick-serve)
![](https://tokei.rs/b1/github/joaofl/quick-serve?category=code)
[![](https://deps.rs/repo/github/joaofl/quick-serve/status.svg)](https://deps.rs/repo/github/joaofl/quick-serve)

![alt text](logo.png "Logo")

# Quick-serve
No setup, zero-config, multi-platform, multi-protocol, standalone server for developers or whoever wants to promptly 
serve some files over the network.

## Motivation

As an embedded software engineer, I routinely encounter the need for seamless file transfers between host and target 
devices in the course of various development tasks. Whether the objective is upgrading a system image, booting a Linux 
Kernel from the bootloader, retrieving packages from remote repositories, fetching a Git repository or sharing files with 
your colleague next desk, the constant requirement is a quick and straightforward file server. The capability to promptly 
set up an FTP, TFTP, or HTTP server proves to be a time-saving and efficient solution in navigating the most diverse 
file exchange scenarios.

I developed this application as an exercise in learning Rust because I couldn't find a solution that seamlessly served 
multiple protocols, was headless, and supported various platforms. Unlike many dedicated servers tailored for either 
Windows or Linux, with or without a UI, my app aims to bridge the gap by offering a versatile, multi-platform, and 
protocol-agnostic solution.

## Dependencies

### Fedora
```sh
sudo dnf install glibc2-devel atk-devel cairo-devel pango-devel gdk-pixbuf2-devel gtk3-devel
```

### Ubuntu
```sh
sudo apt install libatk1.0-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libgtk-3-dev
```

## Install and Run

```sh
cargo install quick-serve
quick-serve
```

## Build and Run

```sh
git clone https://github.com/joaofl/quick-serve.git
cd quick-serve
cargo run --release
```

The `ui` is optional and can be refrained from compilation with:

```sh
cargo build --release --no-default-features
```

Even if it is compiled, the UI can still be disabled at runtime.

## Cross compile:

```sh
sudo dnf install mingw64-gcc
cargo install cross

./cross-build-all
# or
cross build --release
```

## Usage

```shell
Quick-serve

Usage: quick-serve [OPTIONS]

Options:
  -b, --bind-ip=<IP>     Bind IP [default: 127.0.0.1]
  -p, --serve-dir=<DIR>  Path to serve [default: /tmp/]
  -v, --verbose...       Verbose logging
  -H, --http[=<PORT>]    Start the HTTP server [default port: 8080]
  -f, --ftp[=<PORT>]     Start the FTP server [default port: 2121]
  -t, --tftp[=<PORT>]    Start the TFTP server [default port: 6969]
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

## Implementation Goals

### Supported Protocols
- [x] FTP
- [x] HTTP
- [x] TFTP
- [ ] HTTPS
- [ ] DHCP
- [ ] SFTP
- [ ] NFS
- [ ] SAMBA

### Interface
- [x] Command line
- [ ] Local interface
- [ ] Web interface
- [ ] Terminal interface

### Functionalities
- [ ] Serve `n` files and exit
- [ ] Serve for `t` seconds and exit
- [ ] Show number of files being served
- [ ] Report transfer rate
- [ ] Report transferred files
- [ ] Show statistics when exit
- [ ] Color-code logs according to protocol
- [ ] Add log filtering options
- [ ] Refine on each protocol's specific logs

## TODO
- [ ] Fix excessive CPU usage when using the UI

[![Build Status](https://github.com/joaofl/quick-serve/actions/workflows/rust.yml/badge.svg)](https://github.com/joaofl/quick-serve/actions/workflows/rust.yml)
![](https://tokei.rs/b1/github/joaofl/quick-serve?category=code)
[![](https://deps.rs/repo/github/joaofl/quick-serve/status.svg)](https://deps.rs/repo/github/joaofl/quick-serve)


# Quick-serve
No setup, zero-config, multi-platform, multi-protocol, standalone server for developers or whoever wants to promptly 
serve some files over the network.

## Motivation

As an embedded software engineer, I routinely encounter the need for seamless file transfers between host and target 
devices in the course of various development tasks. Whether the objective is upgrading a system image, booting a Linux 
Kernel from the bootloader, retrieving packages from remote repositories, fetching a Git repository or sharing files with 
your colleague next desk, the constant requirement is a quick and straightforward file server. The capability to promptly 
set up an FTP, TFTP, or HTTP server proves to be a time-saving and efficient solution in navigating the most diverse 
embedded systems development scenarios.

I developed this application as an exercise in learning Rust because I couldn't find a solution that seamlessly served 
multiple protocols, was headless, and supported various platforms. Unlike many dedicated servers tailored for either 
Windows or Linux, with or without a UI, my app aims to bridge the gap by offering a versatile, multi-platform, and 
protocol-agnostic solution.

## Usage

```shell
Options:
  -b, --bind-ip=<IP>
          Bind IP [default: 127.0.0.1]

  -p, --serve-dir=<DIR>
          Path to serve
          
          [default: /tmp/]

  -v, --verbose...
          Verbose logging

  -H, --http[=<PORT>]
          Start the HTTP server [default port: 8080]

  -f, --ftp[=<PORT>]
          Start the FTP server [default port: 2121]

  -t, --tftp[=<PORT>]
          Start the TFTP server [default port: 6969]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## Wishlist

- [x] FTP 
- [x] HTTP
- [x] TFTP 
- [ ] DHCP 
- [ ] SFTP 
- [x] Headless version
- [ ] User interface
- [ ] Timout
- [ ] Show transfer rate
- [ ] Color-code logs according to protocol
- [ ] Add logs filter levels and source

## Compiling on Ubuntu 22.04

Build and run:
```bash
cargo build --release
```

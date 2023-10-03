[![Build Status](https://github.com/joaofl/any-serve/actions/workflows/rust.yml/badge.svg)](https://github.com/joaofl/any-serve/actions/workflows/rust.yml)
![](https://tokei.rs/b1/github/joaofl/any-serve?category=code)
[![](https://deps.rs/repo/github/joaofl/any-serve/status.svg)](https://deps.rs/repo/github/joaofl/any-serve)


# Any-serve
No setup, no config-file, multi-platform, multi-protocol, standalone server for developers or whoever whats to quickly server some files on a LAN.
The swiss-knife of file serving for developers.

![image](https://github.com/joaofl/any-serve/assets/8092452/0f6a2e10-64f9-4511-96ce-090aada2415e)

## Wishlist

- [x] FTP 
- [x] HTTP
- [ ] TFTP 
- [ ] SFTP 
- [ ] DHCP 
- [x] Add "Folder chooser" dialog
- [ ] Show transfer rate (per protocol?)
- [ ] Show color-codes for the logs to distinguish between protocols
- [ ] Have a filter to the log levels and sources
- [ ] Headless version

## Compiling (tested on Ubuntu 22.04 and Fedora 19)

On Ubuntu
```bash
sudo apt install libgtk-3-dev
```

Build and run:
```bash
cargo run
```

[![Rust](https://github.com/joaofl/any-serve/actions/workflows/rust.yml/badge.svg)](https://github.com/joaofl/any-serve/actions/workflows/rust.yml)

# Anyserve
No setup, multi-platform, multi-protocol server for developers or whoever whats to quickly server some files.


## Roadmap

- [x] FTP 
- [ ] TFTP 
- [ ] SFTP 
- [ ] DHCP 
- [ ] HTTP(s)
- [x] Add "Folder chooser" dialog
- [ ] Show transfer rate (per protocol?)
- [ ] Show color-codes for the logs to distinguish between protocols
- [ ] Have a filter to the log levels and sources

## Compiling (tested on Ubuntu 22.04 and Fedora 19)

On Ubuntu
```bash
sudo apt install libgtk-3-dev
```

Build and run:
```bash
cargo run
```
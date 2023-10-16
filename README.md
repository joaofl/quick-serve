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
- [\] TFTP 
- [ ] DHCP 
- [ ] SFTP 
- [x] Add "Folder chooser" dialog
- [ ] Headless version
- [ ] Show transfer rate
- [ ] Color-code logs according to protocol
- [ ] Add logs filter levels and source

## Compiling on Ubuntu 22.04

Install dependencies
```bash
sudo apt install libgtk-3-dev
```

Build and run:
```bash
cargo run
```

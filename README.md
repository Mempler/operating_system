# A yet to be named Operating System

![https://matrix.to/#/#osdev-general:peerstudios.net](https://img.shields.io/matrix/osdev-general%3Apeerstudios.net?server_fqdn=matrix.peerstudios.net&logo=matrix&label=Chat%20with%20us!&color=red)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/mempler/operating_system/rust.yml)

**NOTE: This project is still in early development and is not ready for use.**

This is a hobby project of mine to create a simple operating system from scratch. \
The goal is to create a simple operating system that can be used to run simple programs.



## Table of Contents

- [A yet to be named Operating System](#a-yet-to-be-named-operating-system)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
    - [Prerequisites](#prerequisites)
    - [How to Build](#how-to-build)
    - [How to run / test](#how-to-run--test)
  - [Contributing](#contributing)
  - [LICENSE](#license)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [QEMU](https://www.qemu.org/download/)
- [Any Linux Distribution](https://www.linux.org/pages/download/)
- [Git](https://git-scm.com/downloads)

### How to Build

Building is straight forward. Just run the following command:
```
~$ cargo xbuild
```

it will then create a bootable image at `target/disk/disk.img` which can be run using QEMU or physical hardware (TOTALLY NOT RECOMMENDED!!)

### How to run / test

Running / testing is only supported using QEMU and only under Linux.

You can run the following command to run the OS in QEMU:
```
~$ cargo xrun
```

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to contribute.

## LICENSE

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details \
TL;DR: Do whatever you want with it. Just don't blame me if it breaks your computer but credit me if you use it in your project.

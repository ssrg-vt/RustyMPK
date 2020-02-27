# libhermitMPK: Intra-Unikernel Isolation with Intel Memory Protection Keys

For general information about its design principles and implementation, please read the [paper](https://www.ssrg.ece.vt.edu/papers/vee20-mpk.pdf).

This work was implemented on the top of RustyHermit (https://github.com/hermitcore/libhermit-rs)

## Prerequisites
1. Install Rust toolchain with RustyHertmit extensions.
```sh
$ git clone git@github.com:ssrg-vt/rust.git
$ cd rust
$ git checkout hermit
```
To build the toolchain, you need the configuration file `config.toml` in the root directory of the repository. 
A template `config.toml.example` is part of the repository. 
However, default `config.toml` in this repository, which already enable the support of RustyHermit is recommended.
You have only to change the installation path (see variable `prefix` in `config.toml`).

Afterwards you are able to build the toolchain with following command `./x.py install`.
This will take a while (at least 45 minutes).
Afterwards you have to set the environment variable `XARGO_RUST_SRC` to `/installation_path/lib/rustlib/src/rust/src/`.
Please replace installation_path to the location, where you install the toolchain.

2. Install `uhyve`
```sh
# Get our source code.
$ git clone git@github.com:hermitcore/uhyve.git
$ cd uhyve

# Get a copy of the Rust source code so we can rebuild core
# for a bare-metal target.
$ cargo build

# To avoid *sudo* to start uhyve
$ sudo chmod a+rw /dev/kvm
```
## Build
```sh
# Go to the libhermitMPK repo
$ cd libhermitMPK

# build a debug version
$ make

# build a release version
$ make release=1
```

## Run an application
```sh
# To start the rusty_tests application
$ /path_to_uhyve/uhyve -v /path_to_the_unikernel/rusty_tests
```
## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

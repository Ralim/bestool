# BES programming tool

Rough around the edges minimal rust tool to load code into the BES2300 over the uart.

This is built by capturing the traffic from the windows programming tool.
The `programmer.bin` is just from a uart capture of the payload the tool sends.
This file is obviously copyright BES.
The rest of this code & notes is released under MIT licence.

## Usage

At the moment bestool is not being released to crates.io but will be in the future once its a bit more tested.

### Clone the repository locally

```bash
git clone --recursive https://github.com/Ralim/BES-programming-tool.git
```

### Build the tool

To build the tool you will need a rust toolchain setup on your local machine.
[Rustup](https://rustup.rs/) should make this easy if you dont have one.

```bash
cd bestool/bestool
cargo build --release
```

### Run the tool

```
./target/release/bestool read-image --port /dev/ACM0 flashDump.bin
```

Run the tool with `--help` to view available options.

# Iptvnator

An iptvclient that is capable of parsing and reading m3uplaylists of essentially arbitrary sizes _blazingly fast_

## Install

Just clone the repo and run `cargo build --release` to compile the project. Then put it in your `$PATH` or make a shortcut to the binary (target/release/iptvnator_rs)
You will need to install mpv, and have it in your path, otherwise it wont work

## Left to do

- Implement the ctrlc handler so that the program saves watched links before exiting.
- Create a GUI
  - Would be nice to bundle mpv in some form

# Nebulet

[![Join the chat at https://gitter.im/nebulet/nebulet](https://badges.gitter.im/nebulet/nebulet.svg)](https://gitter.im/nebulet/nebulet?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/nebulet/nebulet.svg?branch=master)](https://travis-ci.org/nebulet/nebulet)

## What is Nebulet?

Nebulet is a microkernel that executes WebAssembly modules in ring 0 and a single address space to increase performance. This allows for low context-switch overhead, syscalls just being function calls, and exotic optimizations that simply would not be possible on conventional operating systems. The WebAssembly is verified, and due to a trick used to optimize out bounds-checking, unable to even represent the act of writing or reading outside its assigned linear memory.

The [Cretonne](https://github.com/cretonne/cretonne) compiler is used to compile WebAssembly to native machine code. Once compiled, there is no complex interactions between the application and the runtime (unlike jit compilers, like v8) to reduce surface area for vulnerabilities.

Right now, Nebulet isn't ready to do anything yet, but it'll get there.

## Building & Running

```sh
# install tools
# make sure that `python` is accessible.
$> cargo install xargo
$> rustup component add rust-src
$> cargo install cargo-xbuild
$> cargo install bootimage

# checkout code and associated submodules
$> git clone https://github.com/nebulet/nebulet.git
$> cd nebulet/ && rustup override set nightly
$> git submodule update --init

# compile the kernel
$> bootimage build

# run qemu
$> bootimage run --release -- -serial stdio
```

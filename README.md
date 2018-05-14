# Nebulet

[![Join the chat at https://gitter.im/nebulet/nebulet](https://badges.gitter.im/nebulet/nebulet.svg)](https://gitter.im/nebulet/nebulet?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/nebulet/nebulet.svg?branch=master)](https://travis-ci.org/nebulet/nebulet)

## What is Nebulet?

Nebulet is a microkernel that executes WebAssembly modules instead of ELF binaries. Furthermore, it does so in ring 0 and in the same address space as the kernel, instead of in ring 3. Normally, this would be super dangerous, but WebAssembly is designed to run safely on remote computers, so it can be securely sandboxed without losing performance.

Eventually, once the [Cretonne](https://github.com/cretonne/cretonne) compiler matures, applications running on Nebulet could be faster than their counterparts running on Linux due to syscalls just being function calls, low context-switch overhead, and exotic optimizations that aren't possible on conventional operating systems.

Right now, Nebulet isn't ready to do anything yet, but it'll get there.

## Building & Running

```sh
# install tools
# make sure that `python` is accessible.
$> cargo install xargo
$> cargo install cargo-xbuild
$> cargo install bootimage

# compile the kernel
$> bootimage build

# run qemu
$> bootimage run
```

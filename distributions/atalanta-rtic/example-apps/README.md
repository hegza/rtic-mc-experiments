# Atalanta RTIC examples

This crate contains simple code examples for Atalanta.

## RTIC disclaimer

Currently, the Atalanta fork of the RTIC framework is an early WiP, ONLY supporting hardware tasks WITH shared resources.

## Build instructions

```sh
git submodule update --init
rustup target add riscv32imc-unknown-none-elf
cargo build --examples -Frtl-tb
```

# binutils-strings-rust

## What is that?

This project is a rewrite a [Binutils](https://www.gnu.org/software/binutils/) `strings` utility in Rust (and without `libbfd` dependency).

It is just a personal interest to accomplish the following:

* Compare `Rust` vs `C` performance for the same algorithm.
* Explore the impact of different optimization changes.

## Useful stuff

1. To print shared library dependencies: `ldd target/release/strings`.
2. Verbose run stats (CPU, context switches, memory usage, etc.): `$(which time) --verbose target/release/strings target/debug/strings` (don't confuse with `bash` built-in `time`).
3. To profile app run with stack traces: `perf record -g target/release/strings target/debug/strings > /dev/null` (and `perf report` to display the report).
4. To profile app run with stack traces (dwarf): `perf record --call-graph dwarf target/release/strings target/debug/strings > /dev/null` (and `perf report` to display the report).


NB: `target/debug/strings` is used as input, because this is a big binary (several MBs), which is enough to collect rich statistics.

Only `all+ASCII` performance comparison brings a meaningful result, because the difference is pretty much isolated to the language.

`data section+ASCII` performance comparison run has no meaning, because `Rust` version is using different backend to parse object file and also it is quite fast, so there is impact of app arguments definition load and parsing.

`Unicode` performance comparison run has a little trust, because `C` version crashes on a big file (`target/debug/strings`), so smaller file is used.

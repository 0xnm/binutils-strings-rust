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

## Performance comparison

`strings` version 2.34

### ASCII chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings target/debug/strings` | 646.7 ± 30.2 | 617.0 | 718.2 | 1.00 |
| `strings target/debug/strings` | 696.3 ± 23.1 | 668.0 | 740.5 | 1.08 ± 0.06 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2544
	Voluntary context switches: 2609
	Involuntary context switches: 4

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 2688
	Voluntary context switches: 4715
	Involuntary context switches: 6

### ASCII chars search, only data section(s) scan (in-memory byte array mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -d target/debug/strings` | 139.6 ± 3.9 | 134.3 | 147.1 | 2.27 ± 0.15 |
| `strings -d target/debug/strings` | 61.4 ± 3.8 | 56.3 | 75.7 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 21212
	Voluntary context switches: 558
	Involuntary context switches: 3

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 4832
	Voluntary context switches: 189
	Involuntary context switches: 0

### Unicode chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -u escape target/release/strings` | 263.7 ± 16.0 | 243.0 | 295.5 | 1.70 ± 0.12 |
| `~/binutils-gdb/binutils/strings -Ue target/release/strings` | 155.5 ± 5.9 | 147.1 | 171.6 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2684
	Voluntary context switches: 789
	Involuntary context switches: 0

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 6692
	Voluntary context switches: 1112
	Involuntary context switches: 0


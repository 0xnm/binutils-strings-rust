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

`strings` version 2.38

### ASCII chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings target/debug/strings` | 864.2 ± 57.0 | 797.0 | 986.9 | 1.04 ± 0.08 |
| `strings target/debug/strings` | 831.4 ± 26.2 | 804.1 | 888.2 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2448
	Voluntary context switches: 2924
	Involuntary context switches: 6

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 2572
	Voluntary context switches: 5347
	Involuntary context switches: 4

### ASCII chars search, only data section(s) scan (in-memory byte array mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -d target/debug/strings` | 161.5 ± 4.0 | 154.4 | 168.5 | 3.42 ± 0.21 |
| `strings -d target/debug/strings` | 47.3 ± 2.6 | 42.6 | 55.0 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 23592
	Voluntary context switches: 599
	Involuntary context switches: 1

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 4832
	Voluntary context switches: 113
	Involuntary context switches: 0

### Unicode chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -u escape target/release/strings` | 260.3 ± 18.0 | 233.0 | 302.5 | 1.40 ± 0.12 |
| `strings -Ue target/release/strings` | 185.3 ± 9.8 | 175.8 | 208.7 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2460
	Voluntary context switches: 830
	Involuntary context switches: 1

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 7784
	Voluntary context switches: 1196
	Involuntary context switches: 0


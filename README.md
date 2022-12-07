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
| `target/release/strings target/debug/strings` | 818.1 ± 70.1 | 751.8 | 952.2 | 1.09 ± 0.11 |
| `strings target/debug/strings` | 748.9 ± 34.2 | 723.7 | 839.7 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2468
	Voluntary context switches: 2903
	Involuntary context switches: 3

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 2532
	Voluntary context switches: 5305
	Involuntary context switches: 4

### ASCII chars search, only data section(s) scan (in-memory byte array mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -d target/debug/strings` | 167.4 ± 19.8 | 151.9 | 236.1 | 3.65 ± 0.46 |
| `strings -d target/debug/strings` | 45.8 ± 2.0 | 41.8 | 56.5 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 23456
	Voluntary context switches: 597
	Involuntary context switches: 0

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 4844
	Voluntary context switches: 113
	Involuntary context switches: 0

### Unicode chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -u escape target/release/strings` | 243.1 ± 13.7 | 225.3 | 264.7 | 1.23 ± 0.13 |
| `strings -Ue target/release/strings` | 197.2 ± 18.1 | 169.6 | 233.2 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2472
	Voluntary context switches: 2903
	Involuntary context switches: 4

#### C variant memory usage and context switches

	Average total size (kbytes): 0
	Minor (reclaiming a frame) page faults: 2187
	Voluntary context switches: 789


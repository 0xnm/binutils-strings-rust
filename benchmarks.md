## Performance comparison

`strings` version 2.34

### ASCII chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings target/debug/strings` | 827.8 ± 46.7 | 731.3 | 886.6 | 1.19 ± 0.08 |
| `strings target/debug/strings` | 697.0 ± 23.5 | 672.9 | 749.5 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2480
	Voluntary context switches: 2621
	Involuntary context switches: 4

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 2776
	Voluntary context switches: 4712
	Involuntary context switches: 2

### ASCII chars search, only data section(s) scan (in-memory byte array mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -d target/debug/strings` | 145.9 ± 4.2 | 138.0 | 154.1 | 2.33 ± 0.19 |
| `strings -d target/debug/strings` | 62.6 ± 4.9 | 57.0 | 86.5 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 21384
	Voluntary context switches: 571
	Involuntary context switches: 0

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 4796
	Voluntary context switches: 186
	Involuntary context switches: 1

### Unicode chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -u escape target/release/strings` | 268.8 ± 14.0 | 251.9 | 303.4 | 1.73 ± 0.10 |
| `~/binutils-gdb/binutils/strings -Ue target/release/strings` | 155.3 ± 4.6 | 146.7 | 164.1 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2468
	Voluntary context switches: 802
	Involuntary context switches: 0

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 6784
	Voluntary context switches: 1110
	Involuntary context switches: 0


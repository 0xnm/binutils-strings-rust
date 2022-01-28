## Performance comparison

`strings` version 2.34

### ASCII chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings target/debug/strings` | 737.4 ± 34.2 | 685.9 | 795.9 | 1.05 ± 0.06 |
| `strings target/debug/strings` | 704.9 ± 28.6 | 682.2 | 777.5 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2460
	Voluntary context switches: 2624
	Involuntary context switches: 4

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 2972
	Voluntary context switches: 4715
	Involuntary context switches: 3

### ASCII chars search, only data section(s) scan (in-memory byte array mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -d target/debug/strings` | 146.4 ± 5.8 | 137.3 | 160.7 | 2.27 ± 0.22 |
| `strings -d target/debug/strings` | 64.4 ± 5.7 | 57.6 | 85.4 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 21256
	Voluntary context switches: 573
	Involuntary context switches: 0

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 4788
	Voluntary context switches: 186
	Involuntary context switches: 0

### Unicode chars search, complete file scan (file stream mode)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `target/release/strings -u escape target/release/strings` | 273.9 ± 12.6 | 248.0 | 291.2 | 1.80 ± 0.09 |
| `~/binutils-gdb/binutils/strings -Ue target/release/strings` | 152.4 ± 3.3 | 148.0 | 159.6 | 1.00 |

#### Rust variant memory usage and context switches

	Maximum resident set size (kbytes): 2460
	Voluntary context switches: 804
	Involuntary context switches: 1

#### C variant memory usage and context switches

	Maximum resident set size (kbytes): 6728
	Voluntary context switches: 1112
	Involuntary context switches: 1


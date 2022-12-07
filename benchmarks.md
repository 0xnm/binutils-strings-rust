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


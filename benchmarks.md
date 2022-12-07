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


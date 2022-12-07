#!/usr/bin/env bash
set -e

# Build stuff
echo 'Building release version'
cargo build --release
echo 'Building debug version (will be used as test input)'
cargo build

BENCH_FILE_MODE_FILE="_bench_file.md"
BENCH_DATA_MODE_FILE="_bench_data_file.md"
BENCH_UNICODE_FILE_MODE_FILE="_bench_file_unicode.md"

# Run benchmarks
hyperfine -w 5 'target/release/strings target/debug/strings' 'strings target/debug/strings' --export-markdown $BENCH_FILE_MODE_FILE
hyperfine -w 5 'target/release/strings -d target/debug/strings' 'strings -d target/debug/strings' --export-markdown $BENCH_DATA_MODE_FILE
# strings crashes on big (target/debug/strings) file, so using a smaller file.
# Don't trust the result a lot.
hyperfine -w 5 'target/release/strings -u escape target/release/strings' 'strings -Ue target/release/strings' --export-markdown $BENCH_UNICODE_FILE_MODE_FILE

TIME_RUN_COMMAND="$(which time) --verbose"
EXTRACT_STATS_COMMAND="sed -n -e 10p -e 14p -e 15p"

TIME_OUTPUT_RUST_FILE_MODE=$(${TIME_RUN_COMMAND} target/release/strings target/debug/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})
TIME_OUTPUT_GDB_FILE_MODE=$(${TIME_RUN_COMMAND} strings target/debug/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})

TIME_OUTPUT_RUST_DATA_MODE=$(${TIME_RUN_COMMAND} target/release/strings -d target/debug/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})
TIME_OUTPUT_GDB_DATA_MODE=$(${TIME_RUN_COMMAND} strings -d target/debug/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})

TIME_OUTPUT_RUST_UNICODE_FILE_MODE=$(${TIME_RUN_COMMAND} target/release/strings -u escape target/release/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})
TIME_OUTPUT_GDB_UNICODE_FILE_MODE=$(${TIME_RUN_COMMAND} strings -Ue target/release/strings 2>&1 >/dev/null | ${EXTRACT_STATS_COMMAND})

# Generate README
echo -en '## Performance comparison\n\n' > benchmarks.md
echo -en '`strings` version ' >> benchmarks.md
strings --version | grep -P -o -e "(\d\.\d.*)" >> benchmarks.md
echo -en '\n' >> benchmarks.md
echo -en '### ASCII chars search, complete file scan (file stream mode)\n\n' >> benchmarks.md
cat $BENCH_FILE_MODE_FILE >> benchmarks.md
rm $BENCH_FILE_MODE_FILE
echo -en '\n' >> benchmarks.md
echo -en '#### Rust variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_RUST_FILE_MODE}\n\n" >> benchmarks.md
echo -en '#### C variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_GDB_FILE_MODE}\n\n" >> benchmarks.md
echo -en '### ASCII chars search, only data section(s) scan (in-memory byte array mode)\n\n' >> benchmarks.md
cat $BENCH_DATA_MODE_FILE >> benchmarks.md
rm $BENCH_DATA_MODE_FILE
echo -en '\n' >> benchmarks.md
echo -en '#### Rust variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_RUST_DATA_MODE}\n\n" >> benchmarks.md
echo -en '#### C variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_GDB_DATA_MODE}\n\n" >> benchmarks.md
echo -en '### Unicode chars search, complete file scan (file stream mode)\n\n' >> benchmarks.md
cat $BENCH_UNICODE_FILE_MODE_FILE >> benchmarks.md
rm $BENCH_UNICODE_FILE_MODE_FILE
echo -en '\n' >> benchmarks.md
echo -en '#### Rust variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_RUST_UNICODE_FILE_MODE}\n\n" >> benchmarks.md
echo -en '#### C variant memory usage and context switches\n\n' >> benchmarks.md
echo -en "${TIME_OUTPUT_GDB_UNICODE_FILE_MODE}\n\n" >> benchmarks.md

TEST_FOLDER="test-resources"
TEST_FILE="test-resources/a.out"

strings $TEST_FILE > $TEST_FOLDER/default-output.txt
strings -s $'\n\n' $TEST_FILE > $TEST_FOLDER/output-with-separator.txt
strings -d $TEST_FILE > $TEST_FOLDER/output-datasection.txt
strings -tx $TEST_FILE > $TEST_FOLDER/output-with-address-hex.txt
strings -to $TEST_FILE > $TEST_FOLDER/output-with-address-octal.txt
strings -n8 $TEST_FILE > $TEST_FOLDER/output-with-num-bytes-8.txt
strings -eS $TEST_FILE > $TEST_FOLDER/output-with-encoding-8-bits.txt
strings -f $TEST_FILE > $TEST_FOLDER/output-with-filenames.txt
# TODO once binutils 2.38 released, replace that
~/binutils-gdb/binutils/strings -Ue $TEST_FILE > $TEST_FOLDER/output-with-unicode-escape.txt
~/binutils-gdb/binutils/strings -Ue -tx $TEST_FILE > $TEST_FOLDER/output-with-unicode-escape-address-hex.txt

```sh
cd fuzz/record
mkdir in
cat /dev/random | head -n 1000 > in/input
cargo afl build -r && cargo afl fuzz -i in -o out target/release/record

cd fuzz/sorted_set
mkdir in
cat /dev/random | head -n 1000 > in/input
cargo afl build -r && cargo afl fuzz -i in -o out target/release/sorted_set
```

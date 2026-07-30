```sh
cd fuzz/record
mkdir in
cat /dev/random | head -n 100 > in/input
cargo afl build -r && cargo afl fuzz -i in -o out target/release/record
```

rmdir /s C:\pgo-data
SET RUSTFLAGS=-Cprofile-generate=/pgo-data -Ctarget-cpu=skylake
cargo run --release
C:\Users\Fabian\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-profdata.exe merge -o /pgo-data/merged.profdata /pgo-data
SET RUSTFLAGS=-Cprofile-use=/pgo-data/merged.profdata -Cllvm-args=-pgo-warn-missing-function -Ctarget-cpu=skylake
cargo run --release
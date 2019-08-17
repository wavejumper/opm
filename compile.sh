cargo build --release
cbindgen --config cbindgen.toml --crate opm --output target/release/opm.h
make
mv helloworld.pd_linux target/release

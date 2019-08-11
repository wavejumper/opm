# opm

## Dependencies

Follow the C build instructions for [libpd](https://github.com/libpd/libpd)

```
git clone https://github.com/libpd/libpd.git
cd libpd
git submodule init
git submodule update
make
sudo make install
```


## Building

```
cargo build --release
cbindgen --config cbindgen.toml --crate opm --output target/release/opm.h
make
```

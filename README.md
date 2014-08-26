City Builder in Rust

This is an attempt to follow the [city builder tutorial](https://www.binpress.com/tutorial/creating-a-city-building-game-with-sfml/137) using Rust.

Building and Running

The game uses SFML, so it has to be installed before this game will compile. See the [rust-sfml README](https://github.com/jeremyletang/rust-sfml/blob/master/README.md)
for instructions regarding SFML. Then you just have to get the files and run `cargo build --release` to get the optimized release version in `target/release/`,
or run `cargo run` to just try it out a little. Note that the game must run from the project root to find the `media` folder.

Have fun!
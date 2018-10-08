akafoe-menu
===========

Parser for menus at [www.akafoe.de](http://www.akafoe.de), written in rust.


## Installation

### Dependencies

* quick-xml = "0.12"
* regex = "1.0"
* reqwest = "0.9"
* time = "0.1"

### Build

Compile with cargo to handle dependencies: `$ cargo build --release`
Copy binary to location of choice, e.g. `# cp target/release/akafoe-menu /usr/local/bin`
or install with `carog install --path .` to `~/.cargo/bin` (default).


---

# License

Copyright (c) 2018 Bernd Busse

The MIT License (MIT), see [LICENSE](./LICENSE) for more information

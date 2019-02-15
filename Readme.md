# Nut
--
#### Port of [Bolt DB](https://github.com/boltdb/bolt) in Rust

* **Compatible with Bolt's database format** and provides similar api.

  *Since Nut supports same format as Bolt there are known issues:*
	* Bolt uses Memmap to operate, there is no LRU or other cache, so if database doesn't fit in memory, then it's end of game, Nut will inevitably crash.
	* Data structures are written directly onto disk. No serialization takes place, so Nut doesn't understand db file from computer with different Endianness, for example.
* **Transaction based**. In case of error transaction will be rolled back.
* **Multiple readers, single writer.**
	Nut works just like Bolt: you can have one writer and multiple readers at a time, just be sure they run on different threads. Making writer and reader transaction in same thread will cause deadlock. Writer can write freely if memory map is have enough free pages, in other case it will be waiting for reading transactions to close.

# Usage

Usual way is to `cargo build --release` in console. Docs available via `cargo doc --no-deps --open`. Api is very similar to Bolt if you are familiar with it.

# Nut Bin

Crate also provides `nut` binary which is helpful to inspect database file in various ways. It can be found after `cargo build --release` in `./target/release/nut`.

Excerpt from man:

```
USAGE:
    nut [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check    Runs an exhaustive check to verify that all pages are accessible or are marked as freed.
    dump     Dumps hex of the page
    help     Prints this message or the help of the given subcommand(s)
    info     Prints database info
    pages    Prints a table of pages with their type (Meta, Leaf, Branch, Freelist)
    tree     Prints buckets tree
```

# Disclaimer

I'm not planning to actively mantain this project since it just a hobby project to check out Rust, and there are better alternatives in terms of performance.

--

MIT License
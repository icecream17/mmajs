# mmajs

A metamath verifier and proof assistant, with tactics!

## Usage

```bash
mmajs <database_file> <interface=~tempi>
```

This program will read `<database_file>`. The file `<~tempi>` will be created,
where users edit proofs. [Feedback](https://crates.io/crates/indicatif) and
results will be on the terminal.

Commands will be done in the terminal. Or, you can type:

```
$! <command>
```

inside `<~tempi>` and save. `<~tempi>` is [only processed when saved](https://docs.rs/notify/latest/notify/poll/struct.PollWatcher.html).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

The corresponding SPDX license expression is `(MIT OR Apache-2.0)`.

Note that this is exactly the same license as smetamath-rs (SMM3) and
Metamath-knife, for compatibility reasons.

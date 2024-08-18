# mmajs

A metamath verifier and proof assistant, with tactics!

## Usage

```bash
mmajs <database_file> <interface=~tempi>
```

This program will read `<database_file>`. The file `<~tempi>` will be created, where users edit proofs. [Feedback](https://crates.io/crates/indicatif) and results will be on the terminal.

Commands will be done in the terminal. Or, you can type:

```
$! <command>
```

inside `<~tempi>` and save. `<~tempi>` is [only processed when saved](https://docs.rs/notify/latest/notify/poll/struct.PollWatcher.html).

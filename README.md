## star

A recreation of [Gary Bernhardt's
Selecta](https://github.com/garybernhardt/selecta) written in Rust.

## Usage

```
cargo build --release
cat (ls *.txt | ./target/release/star)
```

See [Selecta's
readme](https://github.com/garybernhardt/selecta/blob/master/README.md) for a
thorough explanation of general use.

## Multiple Selection Mode

Enable multiple selection mode with the `-m` (or `--multiple`) flag. In this
mode, the <kbd>Tab</kbd> key tags or un-tags a line. When multiple lines are
tagged, press <kbd>Alt+Enter</kbd> to print all of them to stdout, joined by
newlines. Pressing <kbd>Enter</kbd> in this mode only prints the currently
selected (_not_ tagged) line to stdout, exactly like in "normal" mode.

## License

BSD 2-clause

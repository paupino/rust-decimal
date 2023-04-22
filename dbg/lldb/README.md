# Rust Decimal LLDB Pretty Printer

This provides a pretty printer for Rust Decimal for use with the LLDB debugger. Setting
this up is relatively straight forward. 

Firstly, copy `decimal_printer.py` to your LLDB script directory. If you don't have one, you could
create one, for example: `~/.lldb/`.

## rust-lldb

From here you can configure the pretty printer by loading the script at runtime, or
by adding the following lines to `~/.lldbinit`:

```
command script import /path/to/decimal_printer.py
type category enable RustDecimal
```

That's it! When your Rust program encounters a Rust Decimal number it will pretty print the value within the debugger.

## Clion


## vscode



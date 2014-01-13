#QuickCheck for Rust

[![Build Status](https://travis-ci.org/lilac/quick-check.png?branch=master)](https://travis-ci.org/lilac/quick-check)

Use `quick_check` to check that a specified property holds
for values of `trait Arbitrary + Shrink`.

Now this library adheres to rustpkg's package structure.
Thus you can conveniently add this line to your program,
```rust
extern mod qc = "github.com/lilac/quick-check";
```
without needing to install this library manually.

##Example
Suppose the current Rust workspace is `~/workspace/rust/`, create a dir for the demo program

    $mkdir -p src/qc-demo

and then write the sample code as follows: `$cat src/qc-demo/main.rs`
```rust
extern mod qc = "github.com/lilac/quick-check";

fn is_sorted<T: TotalOrd>(v: &[T]) -> bool {
    v.windows(2).all(|w| { w[0].cmp(&w[1]).le(&Greater) })
}

fn main() {
    qc::quick_check("sort", qc::config.verbose(true).trials(500),
        |mut v: ~[u8]| { v.sort(); is_sorted(v) });
}
```
Now we build this new pkg with a command:

    $rustpkg build qc-demo

then install the built pkg with:

    $rustpkg install qc-demo

finally the compiled binary qc-demo is in the bin subdir of the current workspace.

## Issues

* Clean up Lazy and Shrink, implement Arbitrary and Shrink further

## License

Copyright License is identical with the Rust project:

    Licensed under the Apache License, Version 2.0
    <LICENSE-APACHE or
    http://www.apache.org/licenses/LICENSE-2.0> or the MIT
    license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
    at your option. All files in the project carrying such
    notice may not be copied, modified, or distributed except
    according to those terms.

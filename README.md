# sims-far

A Rust library to extract data from The Sims 1 UIGraphics.far files.

## Installation

`cargo add sims-far`

## Usage

Extract all contents of the far file:

```rust
use sims_far::Far;
use std::fs::File;

let far = Far::new(r"C:\Program Files (x86)\Maxis\The Sims\UIGraphics\UIGraphics.far");

for manifest_entry in far.manifest.manifest_entries {
    let mut f = File::create(manifest_entry.file_name).unwrap();
    f.write_all(&manifest_entry.get_bytes()).unwrap();
}
```

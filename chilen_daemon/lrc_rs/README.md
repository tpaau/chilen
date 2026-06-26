# lrc_rs

Robust crate for working with synced lyrics content in the LRC format.

## Features
- Reliable parser built with the [`nom`](https://crates.io/crates/nom) crate
- Coverage for all the ID tags and comment tags
- Support for tags from the A2 extension
- Easy serialization
- Simple conversion between synced and unsynced lyrics
- Straightforward access to lyrics data with timestamps

## Examples
Parse some LRC content

Automagically detect whether lyrics are synced or unsynced

Convert synced lyrics to unsynced lyrics

Create synced lyrics

Access lyrics data at a specific timestamp

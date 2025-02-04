## Dependencies
You'll need to place this repo in a directory together with the rust-gpu repo because of some versioning complexities.

## Running
Dev mode: `cargo run`
Release mode (much faster): `cargo run --release`

## Profiling
1. Run with profiling: `cargo run --release --features bevy/trace_chrome`
2. View trace at [perfetto](https://ui.perfetto.dev/)

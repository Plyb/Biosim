# 2025-08-26

1. My current GPU (2060 Super) maxes out at 1024 work group invocations, so trying to do more threads than 32x32 causes device loss (leading to a panic in bevy)
2. Bevy for some reason doesn't like it if I use a rust-gpu shader that includes an enum with repr(u8). Not a big deal though since my cells will eventually be several bytes in size

# 2025-02-03

1. Bevy seems to expect the vertex and fragment shader entry points to be specifically named "vertex" and "fragment" respectively

2. Bevy needs a specific feature enabled to be able to consume spirv shaders (`shader_format_spirv`)

# 2025-02-01
1. You need to run `cargo build` or `cargo run` from the folder that has `rust-toolchain.toml`, but the root `Cargo.toml` has higher priority than the folder `Cargo.toml`s even then. In practice this means that the `[patch.crates-io]` declaration needs to be in the root `Cargo.toml`

2. At the time of writing this, the latest releasted version of `spirv-builder` is 0.9, but that one requires an older toolchain (`2023-05-27`), which is incompatible with many newer libraries. In particular, `ahash` and `elsa` have been problematic. You can sometimes lock them to an older version using `cargo update -p <package-name> --precise <version>`. However, wgpu in particular is problematic here, because the most up-to-date version of wgpu that works with `2023-05-27` transitively relies on a version of ahash that *doesn't* work with that version.

3. You can use `[patch.crates.io]` in the root `Cargo.toml` (see 1.) to override where cargo loads dependencies. At the time of writing this, the master branch of rust-gpu on github is on toolchain version `2024-11-22` which *is* compatible with wgpu 22.0 (along with its transitive dependencies). Using patches, we can point to the updated but unreleased version from github

4. Just using patches leads to an error saying "library limit of 65535 objects exceeded". From what I'm seeing online, this is a limitation with the built-in linker used on windows (which comes from the msvc tools). The solution was to add a `.cargo/config.toml`. Some suggestions said that just using lld-link (LLVM's linker) or rust-lld (seems to come packaged with rust) would work, but it didn't work for me until I also added the `rustflags = ["-Zshare-generics=off"]` line. I don't really know what it does, but it's working now.

5. Similar to 1., profiles don't work except for in the root `Cargo.toml`.
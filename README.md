# assert-env

A lean, zero-dependency Rust utility for runtime environment variable assertions.

## Features

- **Validation**: Ensure environment variables exist and match expected types.
- **Types**: Supports `str`, `int`, `float`, `bool`, and `any`.
- **Lean**: No external crates, uses only the Rust standard library.
- **Transparent**: On Unix, replaces itself with the target process via `exec`.

## Usage

```bash
assert-env [-f path/to/config.toml] "<command>"
```

Example:
```bash
assert-env "node index.js"
```

## Configuration (AssertEnv.toml)

```toml
[required]
DB_HOST = "str"
DB_PORT = "int"

[optional]
TIMEOUT = "float"
DEBUG = "any"
```

- **required**: Must exist and be non-empty.
- **optional**: If present, must match the specified type.

## Installation

### From Crates.io

The easiest way to install `assert-env` is from [crates.io](https://crates.io):

```bash
cargo install assert-env
```

### From Source

If you want to build and install it from the cloned repository, you can use the `--path` flag. This command compiles the local source code and places the resulting binary in your Cargo binary directory (usually `~/.cargo/bin`):

```bash
cargo install --path .
```

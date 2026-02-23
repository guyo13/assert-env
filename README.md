# assert-env

A lean, zero-dependency Rust utility for runtime environment variable assertions.

## Features

- **Validation**: Ensure environment variables exist and match expected types.
- **Types**: Supports `str`, `int`, `float`, and `any`.
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

```bash
cargo install --path .
```

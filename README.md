# docments

`docments` adds Rust doc comments to function parameters and makes the documentation readable at runtime. Put `#[docments]` on a function, an impl block, or a struct. Write `///` comments above its parameters or fields.

The crate records each item's documentation, parameter types and documentation, and return type in a global registry. At runtime, you can look up an item and display its documentation in the layout of a Python `doc()` call.

Rust allows `///` comments on most items. Procedural macros can read them during compilation. rustc rejects these comments on function parameters. `#[docments]` reads and removes them before that check. It adds them to the function's rustdoc as an argument list, which appears in `cargo doc`.

## Use

```rust
use docments::docments;

#[docments]
/// Restart the kernel `kid`, keeping its id.
fn restart(
    /// Kernel id
    kid: &str,
    /// Wait until ready?
    wait: bool,
) -> String { format!("{kid}:{wait}") }

let d = docments::find("restart").unwrap();
print!("{d}");
```

This prints:

```text
fn restart(
    kid: &str, // Kernel id
    wait: bool, // Wait until ready?
) -> String
Restart the kernel `kid`, keeping its id.
```

`docments::index()` gives one line per documented item, grouped by module:

```text
# module myapp
- fn ping()
- fn restart(kid: &str, wait: bool) -> String  # Restart the kernel `kid`, keeping its id.
```

## Where the attribute goes

Place `#[docments]` on one of these items:

- A function. The macro skips receivers such as `self`. A parameter's `#[cfg]` condition also controls its registry entry.
- An impl block. Each method has a registry entry named `Type::method`. Do not put the attribute on individual methods. The generated static cannot appear inside an `impl`.
- A struct. Its fields become the documented parameters. The macro leaves field comments in place because Rust already allows them.

Write each parameter on its own line, with its `///` comments above it. Macros cannot read ordinary `//` comments. A trailing `///` comment attaches to the next parameter.

Use rustfmt's default `fn_params_layout`. It keeps one parameter per line when any parameter has a doc comment. Avoid the `Compressed` layout in crates using docments. It puts undocumented parameters together on one line.

## Runtime API

Use these functions to query the registry:

- `docments::all()` returns every registered item, sorted by module then name.
- `docments::find(name)` finds an item by full path, such as `myapp::restart`. It also accepts a unique name or trailing path. For example, `stop` finds `Gate::stop` when that match is unique.
- `docments::index()` returns the module-grouped listing above.

`Docments` implements `Display` in the layout above. Its `summary()` returns the first paragraph as rustdoc reads it. `path()` returns the item's path. `sig()` returns its one-line signature. Its fields are public.

The macro also creates a hidden `pub static` for direct reference to each item. For example, `restart` has `RESTART_DOCMENTS` and `Gate::stop` has `GATESTOP_DOCMENTS`.

The registry uses `inventory`. It works in the contexts inventory supports, including ordinary binaries, tests, and libraries linked into them.

## JSON schemas (`schema` feature)

Enable the `schema` Cargo feature to generate LLM tool definitions:

```toml
docments = { version = "0.1", features = ["schema"] }
```

Use `#[docments(schema)]` on a function or impl block to register a schema-generating function. `docments::get_schema(name)` returns the tool definition with `name`, `description`, and `input_schema` fields.

Each parameter's schema comes from [schemars](https://docs.rs/schemars). Its description comes from its doc comment. Parameters typed `Option<T>` are optional. All other parameters are required. The tool description ends with the return type.

Every parameter type must implement `schemars::JsonSchema`. Schema generation is opt-in for each function. Plain `#[docments]` does not require this trait.

## Development

```bash
cargo test
cargo fmt && cargo clippy --all-targets -- -D warnings
```

## Release

```bash
cargo test
ship-release
```

`ship-release` tags the Cargo version and pushes. CI publishes to crates.io through trusted publishing and creates the GitHub release. fastship then bumps `Cargo.toml`.

`docments-macros` has its own version. CI publishes it only when that version is not yet on crates.io. Bump it by hand when the macro changes.

For each crate's first release, run `cargo publish` manually. crates.io requires an existing crate before you can configure its trusted publisher.

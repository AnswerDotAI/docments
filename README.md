# docments

Doc comments on function parameters, readable at runtime. Put `#[docments]` on a fn, an impl block, or a struct, write `///` above each parameter, and the crate records the item's doc, its parameters with their types and docs, and its return type in a global registry, then renders them in the layout of a Python `doc()` call.

Rust already allows `///` on nearly everything, and a proc macro can read those comments while the code compiles. The one gap is function parameters, which rustc rejects. `#[docments]` reads them first, strips them, and folds them into the fn's rustdoc as an argument list, so `cargo doc` still shows them.

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

prints

```text
fn restart(
    kid: &str, // Kernel id
    wait: bool, // Wait until ready?
) -> String
Restart the kernel `kid`, keeping its id.
```

and `docments::index()` gives one line per documented item, grouped by module:

```text
# module myapp
- fn ping()
- fn restart(kid: &str, wait: bool) -> String  # Restart the kernel `kid`, keeping its id.
```

## Where the attribute goes

- On a fn. Receivers (`self`) are skipped. A `#[cfg]` on a parameter gates its registry entry the same way.
- On an impl block, not on its methods: a static cannot live inside an `impl`. Each method is recorded as `Type::method`.
- On a struct. Field docs are legal Rust already, so nothing is stripped; the fields become the params.

Param docs are the `///` lines above the parameter, one parameter per line. Trailing `// comments` are not seen by macros, and a trailing `///` would attach to the next parameter, so neither works. rustfmt's default `fn_params_layout` keeps one parameter per line once any of them carries a doc; the `Compressed` layout packs undocumented ones together, so avoid it in crates that use docments.

## Runtime API

- `docments::all()`: every registered item, sorted by module then name.
- `docments::find(name)`: by full path (`myapp::restart`), or by a name or trailing path that is unique (`stop` finds `Gate::stop`).
- `docments::index()`: the module-grouped listing above.
- `Docments` implements `Display` in the layout above, and has `summary()` (first doc line), `path()`, and `sig()` (the one-line signature). Its fields are public.
- Each item also gets a hidden `pub static`, `RESTART_DOCMENTS` for `restart` and `GATESTOP_DOCMENTS` for `Gate::stop`, for direct reference.

The registry is `inventory`, so it works wherever inventory does: ordinary binaries, tests, and libraries linked into them.

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

`ship-release` tags the Cargo version and pushes. CI publishes to crates.io via trusted publishing and creates the GitHub release, then fastship bumps `Cargo.toml`. `docments-macros` has its own version and is published only when that version is not on crates.io yet, so bump it by hand when the macro changes. The first release of each crate needs a manual `cargo publish`, since crates.io only lets a trusted publisher be configured for a crate that already exists.

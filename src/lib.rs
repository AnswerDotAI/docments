//! Doc comments on function parameters, readable at runtime.
//!
//! Put `#[docments]` on a fn, an impl block, or a struct. The item's doc comment and the `///` comment above each
//! parameter are recorded in a registry, and the parameter comments are also folded into the item's rustdoc.
//!
//! ```
//! use docments::docments;
//!
//! #[docments]
//! /// Restart the kernel `kid`, keeping its id.
//! fn restart(
//!     /// Kernel id
//!     kid: &str,
//!     /// Wait until ready?
//!     wait: bool,
//! ) -> String { format!("{kid}:{wait}") }
//!
//! let d = docments::find("restart").unwrap();
//! assert_eq!(d.params[0].doc, "Kernel id");
//! print!("{d}");                    // the signature, one param per line with its doc, then the fn's doc
//! print!("{}", docments::index());  // one line per documented item, grouped by module
//! ```
use std::fmt;

pub use docments_macros::docments;
#[doc(hidden)]
pub use inventory;

/// One documented parameter or field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    /// The parameter name, or its pattern, such as `Path(kid)`
    pub name: &'static str,
    /// The type, as rustfmt would print it
    pub ty: &'static str,
    /// Its doc comment; empty when there was none
    pub doc: &'static str,
}

/// The docments of one fn, method, or struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Docments {
    /// `"fn"` or `"struct"`
    pub kind: &'static str,
    /// `restart`, `Gate::stop`, `Query`
    pub name: &'static str,
    /// The `module_path!()` of the declaration
    pub module: &'static str,
    /// The item's own doc comment, lines joined by `\n`; empty when there was none
    pub doc: &'static str,
    /// Parameters (receivers excluded) or fields, in declaration order
    pub params: &'static [Param],
    /// The return type as rustfmt would print it; empty when there is none, and for structs
    pub ret: &'static str,
}

inventory::collect!(&'static Docments);

impl Docments {
    /// The first paragraph of `doc`, its lines joined with spaces: rustdoc's summary convention.
    pub fn summary(&self) -> String {
        self.doc.split("\n\n").next().unwrap_or("").lines().collect::<Vec<_>>().join(" ")
    }

    /// `module::name`.
    pub fn path(&self) -> String {
        format!("{}::{}", self.module, self.name)
    }

    /// The one-line signature: `fn restart(kid: &str, wait: bool) -> String`, or `struct Query { path: String }`.
    pub fn sig(&self) -> String {
        let ps: Vec<String> = self.params.iter().map(|p| format!("{}: {}", p.name, p.ty)).collect();
        match self.kind {
            "struct" if ps.is_empty() => format!("struct {}", self.name),
            "struct" => format!("struct {} {{ {} }}", self.name, ps.join(", ")),
            _ if self.ret.is_empty() => format!("fn {}({})", self.name, ps.join(", ")),
            _ => format!("fn {}({}) -> {}", self.name, ps.join(", "), self.ret),
        }
    }
}

/// The signature, one param per line with its doc as a trailing comment, then the item's doc comment.
impl fmt::Display for Docments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.params.is_empty() {
            writeln!(f, "{}", self.sig())?;
        } else {
            let (open, close) = if self.kind == "struct" { (" {", "}") } else { ("(", ")") };
            writeln!(f, "{} {}{open}", self.kind, self.name)?;
            for p in self.params {
                write!(f, "    {}: {},", p.name, p.ty)?;
                if p.doc.is_empty() { writeln!(f)? } else { writeln!(f, " // {}", p.doc.replace('\n', " "))? }
            }
            if self.ret.is_empty() { writeln!(f, "{close}")? } else { writeln!(f, "{close} -> {}", self.ret)? }
        }
        if !self.doc.is_empty() {
            writeln!(f, "{}", self.doc)?
        }
        Ok(())
    }
}

/// Every registered item, sorted by module then name.
pub fn all() -> Vec<&'static Docments> {
    let mut v: Vec<_> = inventory::iter::<&'static Docments>.into_iter().copied().collect();
    v.sort_by_key(|d| (d.module, d.name));
    v
}

/// The item at the full path `module::name`, or the unique item whose name (or trailing path) is `name`: `stop` finds `Gate::stop`.
pub fn find(name: &str) -> Option<&'static Docments> {
    let all = all();
    if let Some(d) = all.iter().find(|d| d.path() == name) {
        return Some(d);
    }
    let suffix = format!("::{name}");
    let mut it = all.iter().filter(|d| d.name == name || d.path().ends_with(&suffix));
    match (it.next(), it.next()) {
        (Some(d), None) => Some(d),
        _ => None,
    }
}

/// One line per registered item, grouped under a `# module` heading: `- fn restart(kid: &str, wait: bool) -> String  # summary`.
pub fn index() -> String {
    let mut out = String::new();
    let mut module = "";
    for d in all() {
        if d.module != module {
            module = d.module;
            if !out.is_empty() {
                out.push('\n')
            }
            out.push_str(&format!("# module {module}\n"));
        }
        out.push_str(&format!("- {}", d.sig()));
        let s = d.summary();
        if !s.is_empty() {
            out.push_str(&format!("  # {s}"))
        }
        out.push('\n');
    }
    out
}

#[cfg(feature = "schema")]
pub use schemars;
#[cfg(feature = "schema")]
mod schema;
#[cfg(feature = "schema")]
pub use schema::{FnSchema, get_schema};

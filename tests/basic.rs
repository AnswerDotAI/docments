#![allow(dead_code)]
use docments::docments;
use std::collections::HashMap;

pub struct Path<T>(pub T);

#[docments]
/// Restart the kernel `kid`, keeping its id.
///
/// A second paragraph.
fn restart(
    /// Kernel id
    kid: &str,
    /// Wait until ready?
    wait: bool,
    #[cfg(unix)]
    /// Extra args, unix only
    extra: Option<Vec<String>>,
) -> Result<(), String> {
    let _ = (kid, wait, extra);
    Ok(())
}

/// Docs above the attribute.
#[docments]
async fn handler(
    /// The kernel id from the path
    Path(kid): Path<String>,
    (a, b): (i32, String),
    mut n: u32,
    _: [u8; 4],
) -> String {
    let _ = (a, b);
    n += 1;
    format!("{kid}{n}")
}

#[docments]
fn types<'a>(
    /// Bytes
    buf: &'a mut [u8],
    f: impl Fn(i32) -> i32 + Send,
    g: Box<dyn Fn(&str) -> bool>,
    p: *const u8,
    m: HashMap<String, Vec<u8>>,
    label: &str,
) -> &'a [u8] {
    let _ = (f, g, p, m, label);
    buf
}

#[docments]
fn ping() {}

pub struct Gate;

#[docments]
impl Gate {
    /// Stop the gate.
    pub fn stop(
        &self,
        /// Force?
        force: bool,
    ) -> bool {
        force
    }

    /// Open it.
    pub fn open(&mut self) {}
}

#[docments]
/// Query params for the kernel list.
struct Query {
    /// Only kernels bound to this path
    path: Option<String>,
    limit: u32,
}

mod inner {
    #[docments::docments]
    /// Lives in a nested module.
    pub fn nested(
        /// Some number
        n: u32,
    ) -> u32 {
        n
    }
}

mod other {
    #[docments::docments]
    /// Same name, other module.
    pub fn nested() {}
}

#[test]
fn registry_holds_docs_params_and_return() {
    let d = docments::find("restart").unwrap();
    assert_eq!((d.kind, d.module), ("fn", "basic"));
    assert_eq!(d.doc, "Restart the kernel `kid`, keeping its id.\n\nA second paragraph.");
    assert_eq!(d.summary(), "Restart the kernel `kid`, keeping its id.");
    let ps: Vec<_> = d.params.iter().map(|p| (p.name, p.ty, p.doc)).collect();
    assert_eq!(
        ps,
        [("kid", "&str", "Kernel id"), ("wait", "bool", "Wait until ready?"), ("extra", "Option<Vec<String>>", "Extra args, unix only")]
    );
    assert_eq!(d.ret, "Result<(), String>");
    assert_eq!(docments::find("handler").unwrap().doc, "Docs above the attribute.");
    assert_eq!(HANDLER_DOCMENTS.name, "handler");
}

#[test]
fn types_and_patterns_render_like_rustfmt() {
    let tys: Vec<_> = docments::find("types").unwrap().params.iter().map(|p| p.ty).collect();
    assert_eq!(
        tys,
        ["&'a mut [u8]", "impl Fn(i32) -> i32 + Send", "Box<dyn Fn(&str) -> bool>", "*const u8", "HashMap<String, Vec<u8>>", "&str"]
    );
    let h = docments::find("handler").unwrap();
    let names: Vec<_> = h.params.iter().map(|p| p.name).collect();
    assert_eq!(names, ["Path(kid)", "(a, b)", "mut n", "_"]);
    let tys: Vec<_> = h.params.iter().map(|p| p.ty).collect();
    assert_eq!(tys, ["Path<String>", "(i32, String)", "u32", "[u8; 4]"]);
}

#[test]
fn impl_blocks_and_structs() {
    let d = docments::find("Gate::stop").unwrap();
    assert_eq!(d.name, "Gate::stop");
    assert_eq!(d.params.len(), 1, "the receiver is not a param");
    assert_eq!(GATESTOP_DOCMENTS.doc, "Stop the gate.");
    assert_eq!(docments::find("open").unwrap().params, []);
    let q = docments::find("Query").unwrap();
    assert_eq!(q.kind, "struct");
    assert_eq!(q.sig(), "struct Query { path: Option<String>, limit: u32 }");
    assert_eq!(q.params[1].doc, "");
}

#[test]
fn find_by_name_path_or_unique_suffix() {
    assert_eq!(docments::find("stop").unwrap().name, "Gate::stop");
    assert_eq!(docments::find("basic::inner::nested").unwrap().doc, "Lives in a nested module.");
    assert_eq!(docments::find("inner::nested").unwrap().params[0].name, "n");
    assert!(docments::find("nested").is_none(), "ambiguous across modules");
    assert!(docments::find("nope").is_none());
}

#[test]
fn display_uses_the_doc_layout() {
    assert_eq!(
        docments::find("Gate::stop").unwrap().to_string(),
        "fn Gate::stop(\n    force: bool, // Force?\n) -> bool\nStop the gate.\n"
    );
    assert_eq!(docments::find("ping").unwrap().to_string(), "fn ping()\n");
    assert_eq!(
        docments::find("Query").unwrap().to_string(),
        "struct Query {\n    path: Option<String>, // Only kernels bound to this path\n    limit: u32,\n}\nQuery params for the kernel list.\n"
    );
}

#[test]
fn index_groups_by_module() {
    let idx = docments::index();
    assert!(idx.starts_with("# module basic\n"), "{idx}");
    assert!(idx.contains("- fn ping()\n"), "{idx}");
    assert!(idx.contains("- fn restart(kid: &str, wait: bool, extra: Option<Vec<String>>) -> Result<(), String>  # Restart the kernel `kid`, keeping its id.\n"), "{idx}");
    assert!(idx.contains("\n# module basic::inner\n- fn nested(n: u32) -> u32  # Lives in a nested module.\n"), "{idx}");
    assert_eq!(docments::all().len(), 9);
}

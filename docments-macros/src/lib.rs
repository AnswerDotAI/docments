//! The `#[docments]` attribute. Depend on the `docments` crate, which re-exports it.
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Spacing, TokenStream as Ts, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Error, Expr, Fields, FnArg, ImplItem, Item, Lit, Meta, ReturnType, Signature, parse_macro_input, parse_quote};

/// The doc comment carried by `attrs`: one leading space per line removed, blank edges trimmed.
fn doc_of(attrs: &[Attribute]) -> String {
    let mut lines = vec![];
    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }
        let Meta::NameValue(nv) = &a.meta else { continue };
        let Expr::Lit(el) = &nv.value else { continue };
        let Lit::Str(s) = &el.lit else { continue };
        let v = s.value();
        if v.is_empty() {
            lines.push(String::new())
        }
        for l in v.lines() {
            lines.push(l.strip_prefix(' ').unwrap_or(l).trim_end().to_string())
        }
    }
    lines.join("\n").trim_matches('\n').to_string()
}

const KEYWORDS: &[&str] = &["mut", "dyn", "impl", "const", "unsafe", "extern", "ref", "for", "in", "as", "where"];
const SPACED: &[&str] = &["+", "->", "=", "|", "=>"];
const AFTER: &[&str] = &[",", ";", ":", "+", "->", "=", "|", "=>"];

/// Render a type or pattern's tokens the way rustfmt prints them.
fn tidy(ts: Ts) -> String {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut out = String::new();
    let (mut pk, mut pt) = (' ', String::new());
    let mut i = 0;
    while i < toks.len() {
        let (k, t) = match &toks[i] {
            TokenTree::Ident(x) => ('i', x.to_string()),
            TokenTree::Literal(x) => ('i', x.to_string()),
            TokenTree::Group(g) => {
                let inner = tidy(g.stream());
                let s = match g.delimiter() {
                    Delimiter::Parenthesis => format!("({inner})"),
                    Delimiter::Bracket => format!("[{inner}]"),
                    Delimiter::Brace if inner.is_empty() => "{}".into(),
                    Delimiter::Brace => format!("{{ {inner} }}"),
                    Delimiter::None => inner,
                };
                ('g', s)
            }
            TokenTree::Punct(p) => {
                let mut s = p.as_char().to_string();
                let mut sp = p.spacing();
                while sp == Spacing::Joint {
                    match toks.get(i + 1) {
                        Some(TokenTree::Punct(q)) => {
                            s.push(q.as_char());
                            sp = q.spacing();
                            i += 1;
                        }
                        _ => break,
                    }
                }
                ('p', s)
            }
        };
        let space = match (pk, k) {
            (' ', _) => false,
            ('i', 'i') => true,
            ('i', 'g') => KEYWORDS.contains(&pt.as_str()),
            (_, 'p') => SPACED.contains(&t.as_str()),
            ('p', 'i') => AFTER.contains(&pt.as_str()) || pt == ">",
            ('p', _) => AFTER.contains(&pt.as_str()),
            _ => false,
        };
        if space {
            out.push(' ')
        }
        out.push_str(&t);
        (pk, pt) = (k, t);
        i += 1;
    }
    out
}

fn ident_part(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_uppercase()
}

/// One parameter or field, with any `#[cfg]` it carries so its registry entry is gated the same way.
struct Param {
    name: String,
    ty: String,
    doc: String,
    cfgs: Vec<Attribute>,
}

/// Everything the runtime `Docments` value needs, gathered from one item.
struct Doc {
    kind: &'static str,
    name: String,
    stat: Ident,
    doc: String,
    params: Vec<Param>,
    ret: String,
}

fn field_param(name: String, f: &syn::Field) -> Param {
    Param {
        name,
        ty: tidy(f.ty.to_token_stream()),
        doc: doc_of(&f.attrs),
        cfgs: f.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect(),
    }
}

impl Doc {
    /// Read the fn's docs, and read and strip its params' docs; `prefix` is the impl type for methods.
    fn from_sig(prefix: &str, attrs: &mut Vec<Attribute>, sig: &mut Signature) -> Self {
        let (doc, mut params) = (doc_of(attrs), vec![]);
        for arg in sig.inputs.iter_mut() {
            let FnArg::Typed(pt) = arg else { continue };
            let doc = doc_of(&pt.attrs);
            pt.attrs.retain(|a| !a.path().is_ident("doc"));
            let cfgs = pt.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect();
            params.push(Param { name: tidy(pt.pat.to_token_stream()), ty: tidy(pt.ty.to_token_stream()), doc, cfgs });
        }
        if params.iter().any(|p| !p.doc.is_empty()) {
            attrs.push(parse_quote!(#[doc = ""]));
            attrs.push(parse_quote!(#[doc = " # Arguments"]));
            attrs.push(parse_quote!(#[doc = ""]));
            for p in params.iter().filter(|p| !p.doc.is_empty()) {
                let line = format!(" * `{}` - {}", p.name, p.doc.replace('\n', " "));
                attrs.push(parse_quote!(#[doc = #line]));
            }
        }
        let ret = match &sig.output {
            ReturnType::Default => String::new(),
            ReturnType::Type(_, ty) => tidy(ty.to_token_stream()),
        };
        let fname = sig.ident.to_string();
        let name = if prefix.is_empty() { fname.clone() } else { format!("{prefix}::{fname}") };
        let stat = format_ident!("{}{}_DOCMENTS", ident_part(prefix), fname.to_uppercase());
        Doc { kind: "fn", name, stat, doc, params, ret }
    }

    fn from_struct(s: &syn::ItemStruct) -> Self {
        let params = match &s.fields {
            Fields::Named(f) => f.named.iter().map(|f| field_param(f.ident.as_ref().unwrap().to_string(), f)).collect(),
            Fields::Unnamed(f) => f.unnamed.iter().enumerate().map(|(i, f)| field_param(i.to_string(), f)).collect(),
            Fields::Unit => vec![],
        };
        let name = s.ident.to_string();
        let stat = format_ident!("{}_DOCMENTS", name.to_uppercase());
        Doc { kind: "struct", name, stat, doc: doc_of(&s.attrs), params, ret: String::new() }
    }

    /// The hidden static holding this item's docments, and its registration.
    fn emit(&self) -> Ts {
        let Doc { kind, name, stat, doc, ret, .. } = self;
        let params = self
            .params
            .iter()
            .map(|Param { name, ty, doc, cfgs }| quote! { #(#cfgs)* ::docments::Param { name: #name, ty: #ty, doc: #doc } });
        quote! {
            #[doc(hidden)]
            pub static #stat: ::docments::Docments = ::docments::Docments {
                kind: #kind, name: #name, module: module_path!(), doc: #doc,
                params: &[ #(#params),* ],
                ret: #ret,
            };
            ::docments::inventory::submit! { &#stat }
        }
    }
}

/// Record doc comments on a fn, the methods of an impl block, or the fields of a struct, for reading at runtime.
#[proc_macro_attribute]
pub fn docments(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new_spanned(Ts::from(attr), "#[docments] takes no arguments").to_compile_error().into();
    }
    let mut item = parse_macro_input!(item as Item);
    let docs = match &mut item {
        Item::Fn(f) => vec![Doc::from_sig("", &mut f.attrs, &mut f.sig)],
        Item::Impl(im) => {
            let prefix = tidy(im.self_ty.to_token_stream());
            im.items
                .iter_mut()
                .filter_map(|it| if let ImplItem::Fn(m) = it { Some(Doc::from_sig(&prefix, &mut m.attrs, &mut m.sig)) } else { None })
                .collect()
        }
        Item::Struct(s) => vec![Doc::from_struct(s)],
        other => return Error::new_spanned(other, "#[docments] goes on a fn, an impl block, or a struct").to_compile_error().into(),
    };
    let statics = docs.iter().map(Doc::emit);
    quote! { #item #(#statics)* }.into()
}

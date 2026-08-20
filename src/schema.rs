//! JSON schemas for documented fns, in the shape LLM tool-use APIs take.
use crate::Docments;
use schemars::{Schema, SchemaGenerator};
use serde_json::{Map, Value, json};

/// The schema thunk registered by `#[docments(schema)]`: one entry per fn, produced at compile time.
pub struct FnSchema {
    /// The fn's docments static
    pub docs: &'static Docments,
    /// Each parameter's name, whether it is required (not `Option`), and its schema
    pub params: fn(&mut SchemaGenerator) -> Vec<(&'static str, bool, Schema)>,
}

inventory::collect!(&'static FnSchema);

impl FnSchema {
    /// `{"name", "description", "input_schema"}`: the fn as a tool definition, docments as the descriptions.
    pub fn schema(&self) -> Value {
        let mut g = SchemaGenerator::default();
        let (mut props, mut req) = (Map::new(), vec![]);
        for (name, required, s) in (self.params)(&mut g) {
            let mut v = s.to_value();
            let doc = self.docs.params.iter().find(|p| p.name == name).map_or("", |p| p.doc);
            if !doc.is_empty() {
                v.as_object_mut().map(|o| o.insert("description".into(), doc.into()));
            }
            props.insert(name.into(), v);
            if required {
                req.push(Value::from(name))
            }
        }
        let mut input = json!({"type": "object", "properties": props});
        if !req.is_empty() {
            input["required"] = req.into();
        }
        let defs = g.take_definitions(false);
        if !defs.is_empty() {
            input["$defs"] = Value::Object(defs);
        }
        let mut desc = self.docs.doc.to_string();
        if !self.docs.ret.is_empty() {
            if !desc.is_empty() {
                desc.push_str("\n\n");
            }
            desc.push_str(&format!("Returns: `{}`", self.docs.ret));
        }
        json!({"name": self.docs.name, "description": desc, "input_schema": input})
    }
}

/// The tool schema for the fn at `name`, matched like [`crate::find`]; None when no `#[docments(schema)]` fn matches.
pub fn get_schema(name: &str) -> Option<Value> {
    let all: Vec<&'static FnSchema> = inventory::iter::<&'static FnSchema>.into_iter().copied().collect();
    if let Some(f) = all.iter().find(|f| f.docs.path() == name) {
        return Some(f.schema());
    }
    let suffix = format!("::{name}");
    let mut it = all.iter().filter(|f| f.docs.name == name || f.docs.path().ends_with(&suffix));
    match (it.next(), it.next()) {
        (Some(f), None) => Some(f.schema()),
        _ => None,
    }
}

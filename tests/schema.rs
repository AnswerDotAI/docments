use docments::docments;
use schemars::JsonSchema;
use serde_json::json;

/// A place to look for weather.
#[derive(JsonSchema)]
pub struct Spot {
    /// Latitude in degrees
    pub lat: f64,
    /// Longitude in degrees
    pub lon: f64,
}

#[docments(schema)]
/// Fetch a forecast.
///
/// Detail that stays out of the summary.
fn forecast(
    /// Where to forecast
    spot: Spot,
    /// How many days ahead
    days: u32,
    /// Units label
    units: Option<String>,
) -> String {
    format!("{}:{days}:{units:?}", spot.lat)
}

#[test]
fn schema_merges_docments() {
    assert_eq!(forecast(Spot { lat: 1.0, lon: 2.0 }, 3, None), "1:3:None");
    let s = docments::get_schema("forecast").unwrap();
    assert_eq!(s["name"], "forecast");
    assert_eq!(s["description"], "Fetch a forecast.\n\nDetail that stays out of the summary.\n\nReturns: `String`");
    let inp = &s["input_schema"];
    assert_eq!(inp["type"], "object");
    assert_eq!(inp["required"], json!(["spot", "days"]));
    assert_eq!(
        inp["properties"]["days"],
        json!({"description": "How many days ahead", "type": "integer", "format": "uint32", "minimum": 0})
    );
    assert_eq!(inp["properties"]["units"]["type"], json!(["string", "null"]));
    assert_eq!(inp["properties"]["spot"]["$ref"], "#/$defs/Spot");
    assert_eq!(inp["$defs"]["Spot"]["properties"]["lat"]["description"], "Latitude in degrees");
    assert!(docments::get_schema("no_such").is_none());
    assert!(docments::find("forecast").is_some(), "plain docments registered too");
}

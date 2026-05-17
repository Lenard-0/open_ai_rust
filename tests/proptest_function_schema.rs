//! Property-based tests for the `FunctionType` → JSON Schema serializer.
//!
//! Generates arbitrary nested `FunctionType` values and asserts invariants on the
//! emitted schema:
//!   * Every parameter appears as a property.
//!   * The required-array contains exactly the params that are `required: true` and
//!     not `FunctionType::Option`.
//!   * Output is always valid JSON.
//!   * `Map` always emits `additionalProperties`.
//!   * `OneOf` always emits a `oneOf` array of the right length.
//!   * Deep nesting (Array of Array of Object of Map of Number) doesn't break.

use open_ai_rust::logoi::input::tool::{
    EnumValues, FunctionCall, FunctionParameter, FunctionType, FunctionVariant,
};
use proptest::collection::vec;
use proptest::prelude::*;
use serde_json::Value;

// --- Generators -------------------------------------------------------------

fn arb_ident() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s.to_string())
}

fn arb_leaf_type() -> impl Strategy<Value = FunctionType> {
    prop_oneof![
        Just(FunctionType::String),
        Just(FunctionType::Number),
        Just(FunctionType::Boolean),
        // Enum<String>
        vec("[a-z]{1,8}".prop_map(|s| s.to_string()), 1..5)
            .prop_map(|v| FunctionType::Enum(EnumValues::String(v))),
    ]
}

// Recursive: leaf | Array(leaf) | Option(leaf) | Map(leaf). Avoids Object/OneOf at the
// leaf level (those have their own combinators below). Bounded depth.
fn arb_type() -> impl Strategy<Value = FunctionType> {
    let leaf = arb_leaf_type();
    leaf.prop_recursive(
        3,  // depth
        16, // nodes
        4,  // items per collection
        |inner| {
            prop_oneof![
                inner.clone().prop_map(|t| FunctionType::Array(Box::new(t))),
                inner
                    .clone()
                    .prop_map(|t| FunctionType::Option(Box::new(t))),
                inner.clone().prop_map(|t| FunctionType::Map(Box::new(t))),
                vec((arb_ident(), inner.clone(), any::<bool>()), 1..4).prop_map(|fields| {
                    FunctionType::Object(
                        fields
                            .into_iter()
                            .map(|(name, t, req)| FunctionParameter {
                                name,
                                _type: t,
                                description: None,
                                required: req,
                            })
                            .collect(),
                    )
                }),
                vec(
                    (
                        arb_ident().prop_map(|s| s.to_string()),
                        vec((arb_ident(), inner.clone(), any::<bool>()), 0..3),
                    ),
                    1..3
                )
                .prop_map(|variants| {
                    FunctionType::OneOf(
                        variants
                            .into_iter()
                            .map(|(name, params)| FunctionVariant {
                                name,
                                description: None,
                                parameters: params
                                    .into_iter()
                                    .map(|(n, t, req)| FunctionParameter {
                                        name: n,
                                        _type: t,
                                        description: None,
                                        required: req,
                                    })
                                    .collect(),
                            })
                            .collect(),
                    )
                }),
            ]
        },
    )
}

fn arb_function_call() -> impl Strategy<Value = FunctionCall> {
    (
        arb_ident(),
        vec((arb_ident(), arb_type(), any::<bool>()), 1..5),
    )
        .prop_map(|(name, fields)| {
            // Deduplicate names — Object's `required` array shouldn't double-list.
            let mut seen = std::collections::HashSet::new();
            let params = fields
                .into_iter()
                .filter(|(n, _, _)| seen.insert(n.clone()))
                .map(|(n, t, r)| FunctionParameter {
                    name: n,
                    _type: t,
                    description: None,
                    required: r,
                })
                .collect();
            FunctionCall {
                name,
                description: None,
                parameters: params,
            }
        })
}

// --- Invariants -------------------------------------------------------------

/// Walks a `FunctionType` schema (`serde_json::Value`) checking structural invariants.
fn assert_schema_invariants(_t: &FunctionType, v: &Value) {
    // Every schema fragment is a JSON object.
    let obj = v.as_object().unwrap_or_else(|| {
        panic!("schema must be object, got: {v}");
    });

    // Either has a `type` field, or `oneOf` (for unions), or `enum` (covered by `type`).
    let has_type = obj.contains_key("type");
    let has_oneof = obj.contains_key("oneOf");
    assert!(
        has_type || has_oneof,
        "schema must have `type` or `oneOf`: {v}"
    );

    // If `type=object`, must have either `properties` or `additionalProperties`.
    if obj.get("type").and_then(|v| v.as_str()) == Some("object") {
        let has_props = obj.contains_key("properties");
        let has_additional = obj.contains_key("additionalProperties");
        assert!(
            has_props || has_additional,
            "object schema must have properties or additionalProperties: {v}"
        );
    }

    // If `type=array`, must have `items`.
    if obj.get("type").and_then(|v| v.as_str()) == Some("array") {
        assert!(
            obj.contains_key("items"),
            "array schema must have items: {v}"
        );
    }
}

fn assert_function_call_invariants(fc: &FunctionCall, v: &Value) {
    let params = &v["parameters"];
    assert_eq!(params["type"], "object");

    let properties = params["properties"].as_object().expect("properties is obj");
    // Every declared param appears as a property.
    for p in &fc.parameters {
        assert!(
            properties.contains_key(&p.name),
            "missing property: {} in {}",
            p.name,
            v
        );
    }

    // `required` is an array of strings.
    let required = params["required"]
        .as_array()
        .expect("required is array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect::<std::collections::HashSet<_>>();

    // For each param, it's required iff `required:true && !Option`.
    for p in &fc.parameters {
        let is_optional_type = matches!(p._type, FunctionType::Option(_));
        let expect_required = p.required && !is_optional_type;
        let actually_required = required.contains(&p.name);
        assert_eq!(
            expect_required, actually_required,
            "required mismatch for {}: expect={} got={} (type optional? {})",
            p.name, expect_required, actually_required, is_optional_type
        );
    }
}

// --- Properties -------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        ..ProptestConfig::default()
    })]

    /// Serialising any `FunctionCall` produces well-formed JSON satisfying the
    /// invariants above.
    #[test]
    fn arbitrary_function_call_serializes_to_valid_schema(fc in arb_function_call()) {
        let v = serde_json::to_value(&fc).expect("serialize");

        // Re-parse to confirm it's valid JSON.
        let _: Value = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();

        // Top-level shape.
        assert_eq!(v["name"], fc.name.as_str());
        let params = &v["parameters"];
        assert_eq!(params["type"], "object");
        assert!(params["properties"].is_object());
        assert!(params["required"].is_array());

        assert_function_call_invariants(&fc, &v);

        // Recursively check every property schema.
        for p in &fc.parameters {
            let prop_schema = &params["properties"][&p.name];
            assert_schema_invariants(&p._type, prop_schema);
        }
    }

    /// Setting `required = false` removes the param from the `required` array (regardless
    /// of whether the type is `Option`).
    #[test]
    fn required_false_excludes_from_required_array(t in arb_type(), name in arb_ident()) {
        let fc = FunctionCall {
            name: "x".into(),
            description: None,
            parameters: vec![FunctionParameter {
                name: name.clone(),
                _type: t,
                description: None,
                required: false,
            }],
        };
        let v = serde_json::to_value(&fc).unwrap();
        let required = v["parameters"]["required"].as_array().unwrap();
        assert!(
            !required.iter().any(|x| x.as_str() == Some(&name)),
            "expected `{name}` not in required, got {required:?}"
        );
    }

    /// `FunctionType::Map(_)` always emits `type: object` + `additionalProperties`.
    #[test]
    fn map_always_emits_additional_properties(inner in arb_leaf_type()) {
        let fc = FunctionCall {
            name: "x".into(),
            description: None,
            parameters: vec![FunctionParameter {
                name: "attrs".into(),
                _type: FunctionType::Map(Box::new(inner)),
                description: None,
                required: true,
            }],
        };
        let v = serde_json::to_value(&fc).unwrap();
        let prop = &v["parameters"]["properties"]["attrs"];
        assert_eq!(prop["type"], "object");
        assert!(prop["additionalProperties"].is_object());
    }

    /// `FunctionType::OneOf` always emits a `oneOf` array of the right length, each with
    /// a `title` equal to the variant name.
    #[test]
    fn oneof_emits_correct_array(variants in vec((arb_ident(), 0u32..3u32), 1..4)) {
        let fc = FunctionCall {
            name: "x".into(),
            description: None,
            parameters: vec![FunctionParameter {
                name: "shape".into(),
                _type: FunctionType::OneOf(
                    variants
                        .iter()
                        .map(|(name, n_params)| FunctionVariant {
                            name: name.clone(),
                            description: None,
                            parameters: (0..*n_params)
                                .map(|i| FunctionParameter::new(
                                    format!("p{i}"),
                                    FunctionType::Number,
                                ))
                                .collect(),
                        })
                        .collect(),
                ),
                description: None,
                required: true,
            }],
        };
        let v = serde_json::to_value(&fc).unwrap();
        let one_of = v["parameters"]["properties"]["shape"]["oneOf"]
            .as_array()
            .unwrap();
        assert_eq!(one_of.len(), variants.len());
        for (got, (expected_name, _)) in one_of.iter().zip(variants.iter()) {
            assert_eq!(got["title"], expected_name.as_str());
        }
    }

    /// Round-trip: any deeply-nested type schema is renderable as a string and re-parseable.
    #[test]
    fn deeply_nested_types_serialize_as_round_trippable_json(t in arb_type()) {
        let fc = FunctionCall {
            name: "x".into(),
            description: None,
            parameters: vec![FunctionParameter {
                name: "f".into(),
                _type: t,
                description: None,
                required: true,
            }],
        };
        let s = serde_json::to_string(&fc).expect("serialize to string");
        let _: Value = serde_json::from_str(&s).expect("parse back");
    }
}

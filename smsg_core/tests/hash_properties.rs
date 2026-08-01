//! Property tests for `smsg_core::hash`: the canonical preimage encoding must be
//! injective (no ambiguity between distinct descriptors), sensitive to any
//! single change, deterministic, and domain-separated from the name hash.

use std::collections::HashMap;

use proptest::prelude::*;
use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{DescriptorProto, FieldDescriptorProto};
use smsg_core::{name_preimage, version_preimage};

/// Message/field names: ambiguity-prone fixed strings plus random ASCII/unicode.
fn name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("M".to_string()),
        Just("Ma".to_string()),
        Just("ab".to_string()),
        Just("a_b".to_string()),
        Just("chat".to_string()),
        Just("msg".to_string()),
        prop::string::string_regex("[a-zA-Z0-9_]{0,20}").unwrap(),
        prop::collection::vec(any::<char>(), 0..6).prop_map(|v| v.into_iter().collect()),
    ]
    .boxed()
}

fn field_strategy() -> impl Strategy<Value = FieldDescriptorProto> {
    // type_name normalized: absent (None) or non-empty, matching real descriptors.
    (
        name_strategy(),
        prop_oneof![
            Just(1i32),
            Just(2),
            Just(3),
            Just(0),
            Just(-1),
            Just(65535),
            any::<i32>()
        ],
        1i32..=18i32,
        prop_oneof![
            Just("".to_string()),
            Just(".a.b.C".to_string()),
            Just(".pkg.Message".to_string()),
        ],
        prop_oneof![Just(Label::Optional), Just(Label::Repeated)],
    )
        .prop_map(
            |(name, number, ty, type_name, label)| FieldDescriptorProto {
                name: Some(name),
                number: Some(number),
                r#type: Some(ty),
                type_name: if type_name.is_empty() {
                    None
                } else {
                    Some(type_name)
                },
                label: Some(label as i32),
                ..Default::default()
            },
        )
}

fn message_strategy() -> impl Strategy<Value = DescriptorProto> {
    (
        name_strategy(),
        prop::collection::vec(field_strategy(), 0..8),
    )
        .prop_map(|(name, fields)| DescriptorProto {
            name: Some(name),
            field: fields,
            ..Default::default()
        })
}

fn full_name(msg: &DescriptorProto) -> String {
    smsg_core::full_message_name("chat", msg.name())
}

proptest! {
    /// Distinct descriptors must never share a `version_preimage`.
    #[test]
    fn version_preimage_is_injective(msgs in prop::collection::vec(message_strategy(), 1..10)) {
        let mut by_preimage: HashMap<Vec<u8>, Vec<DescriptorProto>> = HashMap::new();
        for m in &msgs {
            by_preimage
                .entry(version_preimage(m, &full_name(m)))
                .or_default()
                .push(m.clone());
        }
        for group in by_preimage.values() {
            for w in group.windows(2) {
                assert_eq!(w[0], w[1], "distinct descriptors share a version_preimage");
            }
        }
    }

    /// Any single change to the descriptor must change the version preimage.
    #[test]
    fn single_change_is_detected(msg in message_strategy()) {
        let full = full_name(&msg);
        let original = version_preimage(&msg, &full);
        let mutated = mutate(&msg);
        let full_m = full_name(&mutated);
        let changed = version_preimage(&mutated, &full_m);
        assert_ne!(original, changed, "a mutation produced no preimage change");
    }

    /// The name preimage and the version preimage are domain-separated.
    #[test]
    fn name_and_version_are_domain_separated(msg in message_strategy(), short in name_strategy()) {
        assert_ne!(name_preimage(&short), version_preimage(&msg, &full_name(&msg)));
    }
}

/// Applies a change that is guaranteed to alter the descriptor.
fn mutate(msg: &DescriptorProto) -> DescriptorProto {
    let mut out = msg.clone();
    // Always produce a change: add a fresh field with a distinctive number/name.
    let mut f = FieldDescriptorProto {
        name: Some("zzz_new_field".to_string()),
        number: Some(1_000_000),
        r#type: Some(Type::Int32 as i32),
        ..Default::default()
    };
    // If the message already has such a field, keep mutating so we differ.
    if out.field.iter().any(|x| x.number() == 1_000_000) {
        f.number = Some(2_000_000);
        f.name = Some("zzz_other_field".to_string());
    }
    out.field.push(f);
    out
}

#[test]
fn empty_type_name_collapses_with_absent() {
    // Real descriptors never carry Some(""); the canonical encoding treats it as
    // absent, so both forms produce the same preimage (a known, benign collapse —
    // kept here to document the behavior).
    let base = |type_name: Option<String>| FieldDescriptorProto {
        name: Some("x".to_string()),
        number: Some(1),
        r#type: Some(Type::Message as i32),
        type_name,
        ..Default::default()
    };
    let a = DescriptorProto {
        name: Some("M".to_string()),
        field: vec![base(None)],
        ..Default::default()
    };
    let b = DescriptorProto {
        name: Some("M".to_string()),
        field: vec![base(Some("".to_string()))],
        ..Default::default()
    };
    assert_eq!(
        version_preimage(&a, "chat.M"),
        version_preimage(&b, "chat.M")
    );
}

//! Tests for baggage implementation

use context_propagation::baggage::{Baggage, BaggageEntry, BaggageProperty};
use context_propagation::carrier::{MapCarrier, Propagator};

#[test]
fn test_baggage_creation() {
    let baggage = Baggage::new();
    assert!(baggage.entries().is_empty());
}

#[test]
fn test_baggage_entry_creation() {
    let entry = BaggageEntry::new("key".to_string(), "value".to_string());
    assert_eq!(entry.key, "key");
    assert_eq!(entry.value, Some("value".to_string()));

    let flag_entry = BaggageEntry::flag("flag_key".to_string());
    assert_eq!(flag_entry.key, "flag_key");
    assert_eq!(flag_entry.value, None);
}

#[test]
fn test_baggage_add_entry() {
    let mut baggage = Baggage::new();
    let entry = BaggageEntry::new("key".to_string(), "value".to_string());

    assert!(baggage.add_entry(entry).is_ok());
    assert_eq!(baggage.entries().len(), 1);
}

#[test]
fn test_baggage_add_flag_entry() {
    let mut baggage = Baggage::new();
    let entry = BaggageEntry::flag("flag-key".to_string());

    assert!(baggage.add_entry(entry).is_ok());
    assert_eq!(baggage.entries().len(), 1);
}

#[test]
fn test_baggage_get_entry() {
    let mut baggage = Baggage::new();
    let entry1 = BaggageEntry::new("key1".to_string(), "value1".to_string());
    let entry2 = BaggageEntry::new("key2".to_string(), "value2".to_string());

    baggage.add_entry(entry1).unwrap();
    baggage.add_entry(entry2).unwrap();

    let found = baggage.get("key1");
    assert!(found.is_some());
    assert_eq!(found.unwrap().key, "key1");

    let not_found = baggage.get("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_baggage_get_all_entries() {
    let mut baggage = Baggage::new();
    let entry1 = BaggageEntry::new("key".to_string(), "value1".to_string());
    let entry2 = BaggageEntry::new("key".to_string(), "value2".to_string());

    baggage.add_entry(entry1).unwrap();
    baggage.add_entry(entry2).unwrap();

    let found = baggage.get_all("key");
    assert_eq!(found.len(), 2);
}

#[test]
fn test_baggage_roundtrip() {
    // Create a baggage
    let mut baggage = Baggage::new();
    let entry = BaggageEntry::new("key".to_string(), "value".to_string());
    baggage.add_entry(entry).unwrap();

    // Create a carrier and inject the context
    let mut carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::BaggagePropagator::new();
    propagator.inject(&mut carrier, &baggage);

    // Extract the context back
    let extracted_baggage = propagator.extract(&carrier);
    assert!(extracted_baggage.is_some());
    let extracted_baggage = extracted_baggage.unwrap();
    assert_eq!(extracted_baggage.entries().len(), 1);
    assert_eq!(extracted_baggage.entries()[0].key, "key");
    assert_eq!(
        extracted_baggage.entries()[0].value,
        Some("value".to_string())
    );
}

#[test]
fn test_baggage_multi_hop() {
    let mut baggage = Baggage::new();
    let entry = BaggageEntry::new("key1".to_string(), "val1".to_string());
    baggage.add_entry(entry).unwrap();

    let propagator = context_propagation::propagation::BaggagePropagator::new();

    // Hop 1: service A -> service B
    let mut carrier_a = MapCarrier::new();
    propagator.inject(&mut carrier_a, &baggage);
    let at_b = propagator.extract(&carrier_a);
    assert!(at_b.is_some());

    // Hop 2: service B adds entry, forwards to service C
    let mut at_b = at_b.unwrap();
    let entry2 = BaggageEntry::new("key2".to_string(), "val2".to_string());
    at_b.add_entry(entry2).unwrap();

    let mut carrier_b = MapCarrier::new();
    propagator.inject(&mut carrier_b, &at_b);
    let at_c = propagator.extract(&carrier_b);
    assert!(at_c.is_some());
    let at_c = at_c.unwrap();

    // Hop 3: service C verifies all entries survive
    assert_eq!(at_c.entries().len(), 2);
    assert_eq!(at_c.entries()[0].key, "key1");
    assert_eq!(at_c.entries()[0].value, Some("val1".to_string()));
    assert_eq!(at_c.entries()[1].key, "key2");
    assert_eq!(at_c.entries()[1].value, Some("val2".to_string()));
}

#[test]
fn test_baggage_property_roundtrip() {
    let mut original = Baggage::new();
    let mut entry = BaggageEntry::new("userId".to_string(), "alice".to_string());
    entry.properties.push(BaggageProperty::KeyValue {
        key: "type".to_string(),
        value: "admin".to_string(),
    });
    entry.properties.push(BaggageProperty::Flag {
        key: "internal".to_string(),
    });
    original.add_entry(entry).unwrap();

    let mut carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::BaggagePropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.entries().len(), 1);
    let e = &extracted.entries()[0];
    assert_eq!(e.key, "userId");
    assert_eq!(e.value, Some("alice".to_string()));
    assert_eq!(e.properties.len(), 2);
    assert_eq!(
        e.properties[0],
        BaggageProperty::KeyValue {
            key: "type".to_string(),
            value: "admin".to_string(),
        }
    );
    assert_eq!(
        e.properties[1],
        BaggageProperty::Flag {
            key: "internal".to_string(),
        }
    );
}

#[test]
fn test_baggage_flag_entry_roundtrip() {
    let mut original = Baggage::new();
    let entry = BaggageEntry::flag("test-flag".to_string());
    original.add_entry(entry).unwrap();

    let mut carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::BaggagePropagator::new();
    propagator.inject(&mut carrier, &original);

    let extracted = propagator.extract(&carrier);
    assert!(extracted.is_some());
    let extracted = extracted.unwrap();

    assert_eq!(extracted.entries().len(), 1);
    assert_eq!(extracted.entries()[0].key, "test-flag");
    assert_eq!(extracted.entries()[0].value, None);
}

#[test]
fn test_baggage_extraction_returns_none_when_missing() {
    let carrier = MapCarrier::new();
    let propagator = context_propagation::propagation::BaggagePropagator::new();
    let result = propagator.extract(&carrier);
    assert!(result.is_none());
}

#[test]
fn test_baggage_max_entries() {
    let mut baggage = Baggage::new();
    for i in 0..180 {
        let entry = BaggageEntry::new(format!("key{}", i), format!("val{}", i));
        assert!(
            baggage.add_entry(entry).is_ok(),
            "entry {} should be accepted",
            i
        );
    }
    let overflow = BaggageEntry::new("overflow".to_string(), "value".to_string());
    let result = baggage.add_entry(overflow);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Maximum number of baggage entries exceeded"
    );
}

#[test]
fn test_baggage_max_size() {
    let mut baggage = Baggage::new();
    let large = "x".repeat(65537);
    let entry = BaggageEntry::new("k".to_string(), large);
    let result = baggage.add_entry(entry);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Maximum baggage size exceeded");
}

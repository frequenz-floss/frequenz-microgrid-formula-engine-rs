// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use crate::value_source::{Reading, ValueSource};

#[test]
fn hashmap_source_reads_present_keys_and_unknown_for_missing() {
    let mut map: HashMap<u64, Option<f32>> = HashMap::from([(1, Some(2.0)), (2, None)]);
    assert_eq!(ValueSource::read(&mut map, &1), Reading::Known(Some(2.0)));
    assert_eq!(ValueSource::read(&mut map, &2), Reading::Known(None));
    assert_eq!(ValueSource::read(&mut map, &3), Reading::Unknown);
}

#[test]
fn reading_map_touches_only_present_values() {
    assert_eq!(
        Reading::Known(Some(2)).map(|v| v * 2),
        Reading::Known(Some(4))
    );
    assert_eq!(
        Reading::<i32>::Known(None).map(|v| v * 2),
        Reading::Known(None)
    );
    assert_eq!(Reading::<i32>::Unknown.map(|v| v * 2), Reading::Unknown);
}

#[test]
fn reading_and_then_can_turn_a_value_into_none() {
    assert_eq!(
        Reading::Known(Some(2)).and_then(|v| Some(v * 2)),
        Reading::Known(Some(4))
    );
    assert_eq!(
        Reading::Known(Some(2)).and_then(|_| None::<i32>),
        Reading::Known(None)
    );
    assert_eq!(
        Reading::<i32>::Known(None).and_then(|v| Some(v * 2)),
        Reading::Known(None)
    );
    assert_eq!(
        Reading::<i32>::Unknown.and_then(|_| None::<i32>),
        Reading::Unknown
    );
}

#[test]
fn reading_zip_prefers_unknown_over_none_over_values() {
    assert_eq!(
        Reading::Known(Some(1)).zip(Reading::Known(Some(2))),
        Reading::Known(Some((1, 2)))
    );
    assert_eq!(
        Reading::Known(Some(1)).zip(Reading::<i32>::Known(None)),
        Reading::Known(None)
    );
    assert_eq!(
        Reading::<i32>::Known(None).zip(Reading::<i32>::Unknown),
        Reading::Unknown
    );
    assert_eq!(
        Reading::<i32>::Unknown.zip(Reading::Known(Some(1))),
        Reading::Unknown
    );
}

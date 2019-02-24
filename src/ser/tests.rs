use std::io::Cursor;
use std::cmp::{max, min};

use proptest::prelude::*;

use super::{shorten, Opts};

fn arb_json() -> impl Strategy<Value = json::Value> {
    let leaf = prop_oneof![
        Just(json::Value::Null),
        any::<bool>().prop_map(json::Value::Bool),
        any::<i32>()
            .prop_map(Into::into)
            .prop_map(json::Value::Number),
        "[[:alnum:]]*".prop_map(json::Value::String),
    ];
    leaf.prop_recursive(8, 512, 16, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..10).prop_map(json::Value::Array),
            prop::collection::hash_map("[[:alnum:]]*", inner, 0..10)
                .prop_map(|map| json::Value::Object(map.into_iter().collect())),
        ]
    })
}

prop_compose! {
    fn arb_opts()(
        max_length in prop::option::of(0..1024u32),
        max_depth in prop::option::of(0..16u32)
    ) -> Opts {
        Opts {
            max_length: max_length.unwrap_or(0), 
            max_depth
        }
    }
}

fn run(opts: Opts, data: &str) -> String {
    let mut result = Vec::new();
    let rdr = Cursor::new(data);
    let wtr = Cursor::new(&mut result);
    shorten(opts, rdr, wtr).unwrap();
    String::from_utf8(result).unwrap()
}

fn get_limits(data: impl AsRef<[u8]>) -> (u32, u32) {
    let mut length = 1;
    let mut depth = 0;
    let mut max_depth = depth;
    for &byte in data.as_ref() {
        match byte {
            b'\n' => length += 1,
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            _ => (),
        }
        max_depth = max(depth, max_depth);
    }

    (length, max_depth)
}

proptest! {
    #[test]
    fn identity(value in arb_json()) {
        let data = json::to_string_pretty(&value).unwrap();
        prop_assert_eq!(run(Opts {
            max_depth: None,
            max_length: 0,
        }, &data), data)
    }

    #[test]
    fn exclude(value in arb_json(), opts in arb_opts()) {
        let data = json::to_string_pretty(&value).unwrap();
        let processed = run(opts, &data);

        let (orig_length, orig_depth) = get_limits(&data);
        let (length, depth) = get_limits(&processed);

        match (opts.max_length, opts.max_depth) {
            (0, None) => {
                prop_assert_eq!(length, orig_length);
                prop_assert_eq!(depth, orig_depth)
            },
            (max_length, None) => {
                prop_assert!(length <= min(orig_length, max_length));
                prop_assert!(depth <= orig_depth)
            },
            (0, Some(max_depth)) => {
                prop_assert!(length <= orig_length);
                prop_assert_eq!(depth, min(orig_depth, max_depth + 1));
            },
            (max_length, Some(max_depth)) => {
                prop_assert!(length <= min(orig_length, max_length));
                prop_assert!(depth <= min(orig_depth, max_depth + 1));
            },
        }
    }
}

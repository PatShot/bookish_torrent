pub use hashes::Hashes;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

/// Metainfo files (aka .torrent files) bencoded dictionaries
#[derive(Debug, Clone, Deserialize)]
struct Torrent {
    // Torrent File Structure.
    // AKA metainfo file.
    announce: String,
    info: Info,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Info {
    /// The file name.
    /// In single file case, it's the name of a file,
    /// In multi file case, it's the name of a directory.
    name: String,
    /// piece length
    #[serde(rename = "piece length")]
    piece_length: usize,

    //Each entry of `pieces` is the SHA1 hash of the piece at the corresponding index.
    pieces: Hashes,

    #[serde(flatten)]
    key: Key,
}

/// There is either `length` or `files` as keys, but not both or neither.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum Key {
    SingleFile { length: usize },
    MultiFile { files: File },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

mod hashes {
    use serde::de::{self, Deserialize, Deserializer, Visitor};
    use serde::ser::{Serialize, Serializer};
    use std::fmt;

    #[derive(Clone, Debug)]
    pub struct Hashes(pub Vec<[u8; 20]>);
    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("byte-string with length in multiple of 20.")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }

            Ok(Hashes(
                v.chunks_exact(20)
                    .map(|slice_20| slice_20.try_into().expect("Guaranteed to be len 20"))
                    .collect(),
            ))
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let single_slice = self.0.concat();
            serializer.serialize_bytes(&single_slice)
        }
    }
}

fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some((n, rest)) = encoded_value
                .strip_prefix('i')
                .and_then(|rest| rest.split_once('e'))
                .and_then(|(digits, rest)| {
                    let n = digits.parse::<i64>().ok()?;
                    Some((n, rest))
                })
            {
                return (n.into(), rest);
            }
        }

        Some('l') => {
            let mut list_values = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (v, remainder) = decode_bencoded_value(rest);
                list_values.push(v);
                rest = remainder;
            }
            return (list_values.into(), &rest[1..]);
        }

        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (k, remainder) = decode_bencoded_value(rest);
                let k = match k {
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("dict keys must be strings, not {k:?}");
                    }
                };
                let (v, remainder) = decode_bencoded_value(remainder);
                dict.insert(k, v);
                rest = remainder;
            }
            return (dict.into(), &rest[1..]);
        }

        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (
                        serde_json::Value::String(rest[..len].to_string()),
                        &rest[len..],
                    );
                }
            }
        }
        _ => {
            println!("Not yet implemented block")
        }
    }

    panic!("Unhandled Encoded Value: {}.", encoded_value)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];
    let _out_put = decode_bencoded_value(input);
    println!("1:{}, 2:{}", _out_put.0, _out_put.1);
}

#[cfg(test)]
mod tests {
    use crate::decode_bencoded_value;

    #[test]
    fn check_correct_string() {
        let input = "5:hello";
        let ans = decode_bencoded_value(&input);
        assert_eq!(ans.0, "hello");
    }

    #[test]
    fn check_incorrect_string() {
        let input = "3:hello";
        let ans = decode_bencoded_value(&input);
        assert_ne!(ans.0, "hello")
    }

    #[test]
    fn check_integers() {
        // let input = "i26e";
        // let ans = decode_bencoded_value(&input);
        // assert_eq!(ans, 26);
        let input = [("i26e", 26), ("i-53e", -53)];
        for item in input {
            let ans = decode_bencoded_value(item.0);
            assert_eq!(ans.0, item.1);
        }
    }

    #[test]
    fn check_goodencoded_lists() {
        let input = [
            ("l4:spam4:eggse", ["spam", "eggs"]),
            ("l5:hello5:worlde", ["hello", "world"]),
        ];
        for item in input {
            let ans = decode_bencoded_value(item.0);
            for (i, subitem) in item.1.iter().enumerate() {
                assert_eq!(ans.0[i], **subitem)
            }
        }
        let num_input = "li-2ei-1ei0ei1ei2ee";
        let num_expected = [-2, -1, 0, 1, 2];
        let ans = decode_bencoded_value(num_input);
        for (index, item) in num_expected.iter().enumerate() {
            assert_eq!(ans.0[index], *item);
        }
    }

    // #[test]
    // fn check_nested_lists() {
    //     let input = "l6:nestedl4:spam4:eggsee";
    //     let ans = decode_bencoded_value(input);
    //     assert_eq!(ans.0, ["nested", ["spam", "eggs"]])
    // }
}

use serde_json;
use std::env;

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

        Some('d') => {}

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

    #[test]
    fn check_nested_lists() {
        let input = "l6:nestedl4:spam4:eggsee";
        let ans = decode_bencoded_value(input);
        assert_eq!(ans.0, ["nested", ["spam", "eggs"]])
    }
}

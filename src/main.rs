use std::env;

use serde_json;

fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next() {
        Some('i') => {
            if let Some(n) = encoded_value.strip_prefix('i')
            .and_then(|rest| rest.split_once('e'))
            .and_then(|(digits, _)| digits.parse:: <i64>().ok()) {
                return n.into();
            }
        }

        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(len) = len.parse:: <usize>() {
                    return serde_json::Value::String(rest[..len].to_string());
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
    let args: Vec<String>  = env::args().collect();
    let input = &args[1];
    let _out_put = decode_bencoded_value(input);
    println!("{}", _out_put);
}


#[cfg(test)]
mod tests {
    use crate::decode_bencoded_value;

    #[test]
    fn check_correct_string() {
        let input = "5:hello";
        let ans = decode_bencoded_value(&input);
        assert_eq!(ans, "hello");
    }

    #[test]
    fn check_incorrect_string() {
        let input = "3:hello";
        let ans = decode_bencoded_value(&input);
        assert_ne!(ans, "hello")
    }

    #[test]
    fn check_integers() {
        // let input = "i26e";
        // let ans = decode_bencoded_value(&input);
        // assert_eq!(ans, 26);
        let input = [("i26e", 26), ("i-53e", -53)];
        for item in input {
            let ans = decode_bencoded_value(item.0);
            assert_eq!(ans, item.1);
        }
    }
}
#[cfg(feature = "json-merge")]
use serde_json::Value;

pub fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                } else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }
            return;
        }
    }

    *a = b;
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge() {
        let mut a = serde_json::json!({
        "title": "This is a title",
        "person" : {
            "firstName": "John",
            "lastName": "Doe",
            "shortName": "John",
        },
        "cities":[ "london", "paris" ]
    });

        let b = serde_json::json!({
        "title": "This is another title",
        "person" : {
            "firstName" : "Jane",
            "lastName": null
        },
        "cities":[ "colombo" ]
    });

        merge(&mut a, b);
        println!("{:#}", a);
    }
}

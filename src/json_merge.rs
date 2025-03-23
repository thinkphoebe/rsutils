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
    use serde_json::json;

    #[test]
    fn test_merge_basic() {
        // 基本测试用例
        let mut a = json!({
            "title": "This is a title",
            "person" : {
                "firstName": "John",
                "lastName": "Doe",
                "shortName": "John",
            },
            "cities":[ "london", "paris" ]
        });

        let b = json!({
            "title": "This is another title",
            "person" : {
                "firstName" : "Jane",
                "lastName": null
            },
            "cities":[ "colombo" ]
        });

        merge(&mut a, b);

        let expected = json!({
            "title": "This is another title",
            "person" : {
                "firstName" : "Jane",
                "shortName": "John"
            },
            "cities":[ "colombo" ]
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_nested_objects() {
        // 测试嵌套对象的合并
        let mut a = json!({
            "nested": {
                "field1": "value1",
                "field2": {
                    "subfield1": 123,
                    "subfield2": "abc"
                }
            }
        });

        let b = json!({
            "nested": {
                "field2": {
                    "subfield1": 456,
                    "subfield3": "new"
                }
            }
        });

        merge(&mut a, b);

        let expected = json!({
            "nested": {
                "field1": "value1",
                "field2": {
                    "subfield1": 456,
                    "subfield2": "abc",
                    "subfield3": "new"
                }
            }
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_arrays() {
        // 测试数组的合并（数组会被完全替换）
        let mut a = json!({
            "array": [1, 2, 3],
            "nested_array": [{"key": "value1"}, {"key": "value2"}]
        });

        let b = json!({
            "array": [4, 5, 6],
            "nested_array": [{"key": "new_value"}]
        });

        merge(&mut a, b);

        let expected = json!({
            "array": [4, 5, 6],
            "nested_array": [{"key": "new_value"}]
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_null_values() {
        // 测试 null 值处理（应删除对应的键）
        let mut a = json!({
            "keep": "value",
            "delete": "to be deleted",
            "nested": {
                "keep": "nested value",
                "delete": "nested to be deleted"
            }
        });

        let b = json!({
            "keep": "value",
            "delete": null,
            "nested": {
                "keep": "nested value",
                "delete": null
            }
        });

        merge(&mut a, b);

        let expected = json!({
            "keep": "value",
            "nested": {
                "keep": "nested value"
            }
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_add_new_fields() {
        // 测试添加新字段
        let mut a = json!({
            "existing": "value"
        });

        let b = json!({
            "new_field": "new value",
            "new_object": {
                "nested": "nested value"
            }
        });

        merge(&mut a, b);

        let expected = json!({
            "existing": "value",
            "new_field": "new value",
            "new_object": {
                "nested": "nested value"
            }
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_type_changes() {
        // 测试类型变更
        let mut a = json!({
            "string_to_int": "string value",
            "array_to_object": [1, 2, 3],
            "object_to_string": {
                "key": "value"
            }
        });

        let b = json!({
            "string_to_int": 123,
            "array_to_object": {"key": "value"},
            "object_to_string": "string value"
        });

        merge(&mut a, b);

        let expected = json!({
            "string_to_int": 123,
            "array_to_object": {"key": "value"},
            "object_to_string": "string value"
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_primitives() {
        // 测试基本类型的合并
        let mut a = json!(42);
        let b = json!("string value");

        merge(&mut a, b);
        assert_eq!(a, json!("string value"));

        let mut a = json!(true);
        let b = json!(false);

        merge(&mut a, b);
        assert_eq!(a, json!(false));
    }

    #[test]
    fn test_merge_complex_scenario() {
        // 复杂场景测试
        let mut a = json!({
            "level1": {
                "level2": {
                    "keep": "keep value",
                    "change": "old value",
                    "delete": "delete value"
                },
                "array": [1, 2, 3]
            },
            "primitive": 42,
            "to_null": "will be null"
        });

        let b = json!({
            "level1": {
                "level2": {
                    "keep": "keep value",
                    "change": "new value",
                    "add": "new field",
                    "delete": null
                },
                "array": [4, 5, 6]
            },
            "primitive": "changed to string",
            "to_null": null,
            "new_top_level": {
                "nested": "value"
            }
        });

        merge(&mut a, b);

        let expected = json!({
            "level1": {
                "level2": {
                    "keep": "keep value",
                    "change": "new value",
                    "add": "new field"
                },
                "array": [4, 5, 6]
            },
            "primitive": "changed to string",
            "new_top_level": {
                "nested": "value"
            }
        });

        assert_eq!(a, expected);
    }

    #[test]
    fn test_merge_top_level_array() {
        // 测试最外层是数组的情况
        let mut a = json!([1, 2, 3]);
        let b = json!([4, 5, 6]);

        merge(&mut a, b);
        assert_eq!(a, json!([4, 5, 6]), "数组应该被完全替换");

        // 数组长度不同的情况
        let mut a = json!([1, 2]);
        let b = json!([3, 4, 5]);

        merge(&mut a, b);
        assert_eq!(a, json!([3, 4, 5]), "不同长度的数组应该被完全替换");
    }

    #[test]
    fn test_merge_nested_arrays() {
        // 测试嵌套数组的合并
        let mut a = json!([
            {"id": 1, "tags": ["important", "urgent"]},
            {"id": 2, "tags": ["normal"]}
        ]);

        let b = json!([
            {"id": 1, "tags": ["important", "postponed"]},
            {"id": 3, "tags": ["new"]}
        ]);

        merge(&mut a, b);

        let expected = json!([
            {"id": 1, "tags": ["important", "postponed"]},
            {"id": 3, "tags": ["new"]}
        ]);

        assert_eq!(a, expected, "嵌套数组应被完全替换");
    }

    #[test]
    fn test_merge_array_to_object() {
        // 测试数组到对象的转换
        let mut a = json!([1, 2, 3]);
        let b = json!({"key": "value"});

        merge(&mut a, b);
        assert_eq!(a, json!({"key": "value"}), "数组应该被对象替换");
    }

    #[test]
    fn test_merge_object_to_array() {
        // 测试对象到数组的转换
        let mut a = json!({"key": "value"});
        let b = json!([1, 2, 3]);

        merge(&mut a, b);
        assert_eq!(a, json!([1, 2, 3]), "对象应该被数组替换");
    }

    #[test]
    fn test_merge_empty_arrays() {
        // 测试空数组
        let mut a = json!([]);
        let b = json!([1, 2, 3]);

        merge(&mut a, b);
        assert_eq!(a, json!([1, 2, 3]), "空数组应该被非空数组替换");

        // 反向测试
        let mut a = json!([1, 2, 3]);
        let b = json!([]);

        merge(&mut a, b);
        assert_eq!(a, json!([]), "非空数组应该被空数组替换");
    }

    #[test]
    fn test_merge_mixed_array_elements() {
        // 测试混合类型的数组元素
        let mut a = json!([1, "string", true, {"key": "value"}]);
        let b = json!([null, 42, false, [1, 2, 3]]);

        merge(&mut a, b);
        assert_eq!(a, json!([null, 42, false, [1, 2, 3]]), "混合类型数组应该被完全替换");
    }

    #[test]
    fn test_merge_array_with_null() {
        // 测试包含 null 的数组合并
        let mut a = json!([1, 2, 3]);
        let b = json!([1, null, 3]);

        merge(&mut a, b);
        assert_eq!(a, json!([1, null, 3]), "含 null 的数组应该被正确替换");

        // 注意：数组中的 null 不会导致元素被删除，而是替换为 null
    }
}

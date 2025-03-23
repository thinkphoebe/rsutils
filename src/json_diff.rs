use serde_json::{Map, Value};

pub fn diff(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            let mut result = Map::new();

            for (k, v_b) in obj_b {
                if let Some(v_a) = obj_a.get(k) {
                    if v_a != v_b {
                        if let Some(v_diff) = diff(v_a, v_b) {
                            result.insert(k.clone(), v_diff);
                        }
                    }
                } else {
                    // 如果键在 a 中不存在，加入结果
                    result.insert(k.clone(), v_b.clone());
                }
            }

            if result.is_empty() {
                None
            } else {
                Some(Value::Object(result))
            }
        }
        _ => {
            if a == b {
                None
            } else {
                Some(b.clone())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json_merge::merge;
    use serde_json::json;

    // 验证 merge(a, diff(a, b)) == merge(a, b) 的辅助函数
    fn verify_diff_merge_equality(a: &Value, b: &Value) {
        let mut a1 = a.clone();
        let mut a2 = a.clone();
        if let Some(c) = diff(a, b) {
            merge(&mut a1, b.clone());
            merge(&mut a2, c);
            assert_eq!(a1, a2, "合并结果不一致: merge(a, b) != merge(a, diff(a, b))");
        } else {
            assert_eq!(a, b, "diff return None but a != b");
        }
    }

    #[test]
    fn test_diff_basic() {
        // 基本测试用例
        let a = json!({
            "title": "This is a title",
            "person": {
                "firstName": "John",
                "lastName": "Doe",
                "shortName": "John",
            },
            "cities": ["london", "paris"]
        });

        let b = json!({
            "title": "This is another title",
            "person": {
                "firstName": "Jane",
                "lastName": null
            },
            "cities": ["colombo"]
        });

        let c = diff(&a, &b).unwrap();

        let expected_diff = json!({
            "title": "This is another title",
            "person": {
                "firstName": "Jane",
                "lastName": null
            },
            "cities": ["colombo"]
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_nested_objects() {
        // 测试嵌套对象的差异计算
        let a = json!({
            "nested": {
                "field1": "value1",
                "field2": {
                    "subfield1": 123,
                    "subfield2": "abc"
                }
            },
            "array": [1, 2, 3],
            "simple": "simple_value"
        });

        let b = json!({
            "nested": {
                "field1": "value1",  // 相同
                "field2": {
                    "subfield1": 456,  // 不同
                    "subfield2": "abc"  // 相同
                }
            },
            "array": [4, 5, 6],  // 不同
            "simple": "simple_value",  // 相同
            "new_field": "new_value"  // 新增
        });

        let c = diff(&a, &b).unwrap();

        // 验证 c 只包含 b 与 a 不同的部分
        let expected_diff = json!({
            "nested": {
                "field2": {
                    "subfield1": 456
                }
            },
            "array": [4, 5, 6],
            "new_field": "new_value"
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_null_values() {
        // 测试 null 值处理（用于删除字段）
        let a = json!({
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

        let c = diff(&a, &b).unwrap();

        let expected_diff = json!({
            "delete": null,
            "nested": {
                "delete": null
            }
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_no_changes() {
        // 测试没有变化的情况
        let a = json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });

        let b = json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });

        let c = diff(&a, &b);
        assert_eq!(c, None, "没有差异时应返回 None");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_type_changes() {
        // 测试类型变更
        let a = json!({
            "field": "string value"
        });

        let b = json!({
            "field": 123
        });

        let c = diff(&a, &b).unwrap();
        let expected_diff = json!({
            "field": 123
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_arrays() {
        // 测试数组变更
        let a = json!({
            "array": [1, {"key": "value"}, 3]
        });

        let b = json!({
            "array": [1, {"key": "new_value"}, 3]
        });

        let c = diff(&a, &b).unwrap();
        let expected_diff = json!({
            "array": [1, {"key": "new_value"}, 3]
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_primitives() {
        // 测试基本类型的差异
        // 1. 整数变更
        let a = json!(42);
        let b = json!(99);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b);
        verify_diff_merge_equality(&a, &b);

        // 2. 字符串变更
        let a = json!("old value");
        let b = json!("new value");

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b);
        verify_diff_merge_equality(&a, &b);

        // 3. 布尔值变更
        let a = json!(true);
        let b = json!(false);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b);
        verify_diff_merge_equality(&a, &b);

        // 4. 类型转换
        let a = json!(42);
        let b = json!("42");

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_add_fields() {
        // 测试添加新字段
        let a = json!({
            "existing": "value"
        });

        let b = json!({
            "existing": "value",
            "new_field": "new value",
            "new_object": {
                "nested": "nested value"
            }
        });

        let c = diff(&a, &b).unwrap();
        let expected_diff = json!({
            "new_field": "new value",
            "new_object": {
                "nested": "nested value"
            }
        });

        assert_eq!(c, expected_diff);
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_complex_scenario() {
        // 复杂场景测试
        let a = json!({
            "level1": {
                "level2": {
                    "keep": "keep value",
                    "change": "old value",
                    "delete": "delete value"
                },
                "array": [1, 2, 3]
            },
            "primitive": 42,
            "to_be_deleted": "will be deleted",
            "same": "unchanged"
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
            "to_be_deleted": null,
            "same": "unchanged",
            "new_top_level": {
                "nested": "value"
            }
        });

        let c = diff(&a, &b).unwrap();

        let mut a1 = a.clone();
        let mut a2 = a.clone();

        merge(&mut a1, b.clone());
        merge(&mut a2, c);

        assert_eq!(a1, a2, "复杂场景合并结果不一致");
    }

    #[test]
    fn test_diff_top_level_array() {
        // 测试最外层是数组的情况
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 4]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "不同的数组应完全替换");
        verify_diff_merge_equality(&a, &b);

        // 数组元素相同，无差异
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 3]);

        let c = diff(&a, &b);
        assert_eq!(c, None, "相同的数组应返回 None");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_nested_arrays() {
        // 测试嵌套数组
        let a = json!([
            {"id": 1, "tags": ["important", "urgent"]},
            {"id": 2, "tags": ["normal"]}
        ]);

        let b = json!([
            {"id": 1, "tags": ["important", "postponed"]},
            {"id": 3, "tags": ["new"]}
        ]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "嵌套数组应完全替换");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_array_add_elements() {
        // 测试向数组添加元素
        let a = json!([1, 2]);
        let b = json!([1, 2, 3, 4]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "添加元素的数组应完全替换");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_array_remove_elements() {
        // 测试从数组删除元素
        let a = json!([1, 2, 3, 4]);
        let b = json!([1, 4]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "删除元素的数组应完全替换");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_array_reorder_elements() {
        // 测试数组元素重新排序
        let a = json!([1, 2, 3]);
        let b = json!([3, 1, 2]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "重新排序的数组应完全替换");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_mixed_types() {
        // 测试类型混合变化（对象 <-> 数组）
        let a = json!({"data": [1, 2, 3]});
        let b = json!([{"data": 1}, {"data": 2}]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "类型变化应完全替换");
        verify_diff_merge_equality(&a, &b);

        // 反向测试：数组 -> 对象
        let a = json!([1, 2, 3]);
        let b = json!({"0": 1, "1": 2, "2": 3});

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "数组到对象的变化应完全替换");
        verify_diff_merge_equality(&a, &b);
    }

    #[test]
    fn test_diff_empty_arrays() {
        // 测试空数组
        let a = json!([]);
        let b = json!([1, 2, 3]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "空数组到非空数组应完全替换");
        verify_diff_merge_equality(&a, &b);

        // 反向测试：非空数组 -> 空数组
        let a = json!([1, 2, 3]);
        let b = json!([]);

        let c = diff(&a, &b).unwrap();
        assert_eq!(c, b, "非空数组到空数组应完全替换");
        verify_diff_merge_equality(&a, &b);

        // 两个空数组
        let a = json!([]);
        let b = json!([]);

        let c = diff(&a, &b);
        assert_eq!(c, None, "两个空数组应返回 None");
        verify_diff_merge_equality(&a, &b);
    }
} 

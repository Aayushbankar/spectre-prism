#[cfg(test)]
mod tests {
    use super::*;
    use vrl::value::Value;

    #[test]
    fn test_dynamic_uid_mapping() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("class_uid".to_string(), Value::from(4002));
        map.insert("category_uid".to_string(), Value::from(5));
        map.insert("type_uid".to_string(), Value::from(400201));
        map.insert("srcip".to_string(), Value::from("10.0.0.1"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid, 4002);
        assert_eq!(result.category_uid, 5);
        assert_eq!(result.type_uid, 400201);
    }
}

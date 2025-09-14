//! JSON序列化器

use super::{Serializer, SerializationContext};
use serde::{Deserialize, Serialize};

/// JSON序列化器
pub struct JsonSerializer {
    pretty_print: bool,
}

impl JsonSerializer {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
        }
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
}

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for JsonSerializer {
    type Error = serde_json::Error;

    fn serialize<T: Serialize>(&self, data: &T, context: &SerializationContext) -> Result<Vec<u8>, Self::Error> {
        let result = if context.pretty_print {
            serde_json::to_vec_pretty(data)?
        } else {
            serde_json::to_vec(data)?
        };
        
        Ok(result)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8], _context: &SerializationContext) -> Result<T, Self::Error> {
        let result = serde_json::from_slice(data)?;
        Ok(result)
    }
}

/// JSON序列化工具函数
pub mod json_utils {
    use super::*;
    use crate::EngineResult;
    use serde_json::Value;

    /// 序列化为格式化的JSON字符串
    pub fn to_pretty_json<T: Serialize>(data: &T) -> EngineResult<String> {
        let json = serde_json::to_string_pretty(data)?;
        Ok(json)
    }

    /// 序列化为紧凑的JSON字符串
    pub fn to_compact_json<T: Serialize>(data: &T) -> EngineResult<String> {
        let json = serde_json::to_string(data)?;
        Ok(json)
    }

    /// 从JSON字符串反序列化
    pub fn from_json_str<T: for<'de> Deserialize<'de>>(json: &str) -> EngineResult<T> {
        let data = serde_json::from_str(json)?;
        Ok(data)
    }

    /// 将JSON值转换为特定类型
    pub fn json_value_to<T: for<'de> Deserialize<'de>>(value: Value) -> EngineResult<T> {
        let data = serde_json::from_value(value)?;
        Ok(data)
    }

    /// 将数据转换为JSON值
    pub fn to_json_value<T: Serialize>(data: &T) -> EngineResult<Value> {
        let value = serde_json::to_value(data)?;
        Ok(value)
    }

    /// 合并两个JSON对象
    pub fn merge_json_objects(mut base: Value, overlay: Value) -> Value {
        if let (Value::Object(ref mut base_map), Value::Object(overlay_map)) = (&mut base, overlay) {
            for (key, value) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_value) if base_value.is_object() && value.is_object() => {
                        *base_value = merge_json_objects(base_value.clone(), value);
                    }
                    _ => {
                        base_map.insert(key, value);
                    }
                }
            }
        }
        base
    }

    /// 提取JSON对象中的字段
    pub fn extract_field<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                Value::Object(map) => {
                    current = map.get(part)?;
                }
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        current = arr.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// 设置JSON对象中的字段
    pub fn set_field(value: &mut Value, path: &str, new_value: Value) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return false;
        }

        let mut current = value;
        
        // 导航到最后一级之前
        for part in &parts[..parts.len() - 1] {
            match current {
                Value::Object(map) => {
                    let part_key = part.to_string();
                    if !map.contains_key(&part_key) {
                        map.insert(part_key.clone(), Value::Object(serde_json::Map::new()));
                    }
                    current = map.get_mut(&part_key).unwrap();
                    if current.is_null() {
                        *current = Value::Object(serde_json::Map::new());
                    }
                }
                Value::Array(arr) => {
                    if let Ok(index) = part.parse::<usize>() {
                        if let Some(item) = arr.get_mut(index) {
                            current = item;
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        // 设置最终值
        let final_key = parts[parts.len() - 1];
        match current {
            Value::Object(map) => {
                map.insert(final_key.to_string(), new_value);
                true
            }
            Value::Array(arr) => {
                if let Ok(index) = final_key.parse::<usize>() {
                    if index < arr.len() {
                        arr[index] = new_value;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 验证JSON模式
    pub fn validate_schema(data: &Value, schema: &Value) -> Vec<String> {
        let mut errors = Vec::new();
        validate_recursive(data, schema, "", &mut errors);
        errors
    }

    fn validate_recursive(data: &Value, schema: &Value, path: &str, errors: &mut Vec<String>) {
        match schema {
            Value::Object(schema_obj) => {
                if let Some(type_value) = schema_obj.get("type") {
                    if let Some(expected_type) = type_value.as_str() {
                        let actual_type = match data {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                        };

                        if actual_type != expected_type {
                            errors.push(format!("Type mismatch at {}: expected {}, got {}", path, expected_type, actual_type));
                        }
                    }
                }

                // 验证对象属性
                if let (Value::Object(data_obj), Some(properties)) = (data, schema_obj.get("properties")) {
                    if let Value::Object(props_schema) = properties {
                        for (key, prop_schema) in props_schema {
                            let new_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                            if let Some(prop_data) = data_obj.get(key) {
                                validate_recursive(prop_data, prop_schema, &new_path, errors);
                            } else if schema_obj.get("required")
                                .and_then(|r| r.as_array())
                                .map(|arr| arr.iter().any(|v| v.as_str() == Some(key)))
                                .unwrap_or(false) {
                                errors.push(format!("Required property missing: {}", new_path));
                            }
                        }
                    }
                }

                // 验证数组项
                if let (Value::Array(data_arr), Some(items_schema)) = (data, schema_obj.get("items")) {
                    for (i, item) in data_arr.iter().enumerate() {
                        let new_path = format!("{}[{}]", path, i);
                        validate_recursive(item, items_schema, &new_path, errors);
                    }
                }
            }
            _ => {
                // 简单类型比较
                if data != schema {
                    errors.push(format!("Value mismatch at {}", path));
                }
            }
        }
    }

    /// 压缩JSON（移除空白）
    pub fn minify_json(json: &str) -> EngineResult<String> {
        let value: Value = serde_json::from_str(json)?;
        let minified = serde_json::to_string(&value)?;
        Ok(minified)
    }

    /// 格式化JSON
    pub fn prettify_json(json: &str) -> EngineResult<String> {
        let value: Value = serde_json::from_str(json)?;
        let pretty = serde_json::to_string_pretty(&value)?;
        Ok(pretty)
    }

    /// 统计JSON大小
    pub fn json_stats(value: &Value) -> JsonStats {
        let mut stats = JsonStats::default();
        count_recursive(value, &mut stats);
        stats
    }

    fn count_recursive(value: &Value, stats: &mut JsonStats) {
        match value {
            Value::Object(obj) => {
                stats.objects += 1;
                stats.total_nodes += 1;
                for (_, v) in obj {
                    count_recursive(v, stats);
                }
            }
            Value::Array(arr) => {
                stats.arrays += 1;
                stats.total_nodes += 1;
                for v in arr {
                    count_recursive(v, stats);
                }
            }
            Value::String(_) => {
                stats.strings += 1;
                stats.total_nodes += 1;
            }
            Value::Number(_) => {
                stats.numbers += 1;
                stats.total_nodes += 1;
            }
            Value::Bool(_) => {
                stats.booleans += 1;
                stats.total_nodes += 1;
            }
            Value::Null => {
                stats.nulls += 1;
                stats.total_nodes += 1;
            }
        }
    }

    /// JSON统计信息
    #[derive(Debug, Default, Clone)]
    pub struct JsonStats {
        pub total_nodes: usize,
        pub objects: usize,
        pub arrays: usize,
        pub strings: usize,
        pub numbers: usize,
        pub booleans: usize,
        pub nulls: usize,
    }
}

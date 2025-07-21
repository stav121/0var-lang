//! Runtime value types for the zvar virtual machine

use crate::error::{ZvarError, ZvarResult};
use std::fmt;

/// Runtime values in the zvar VM
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
}

impl Value {
    /// Get integer value, return error if not an integer
    pub fn as_int(&self) -> ZvarResult<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            Value::Str(_) => Err(ZvarError::runtime("Expected integer, found string")),
            Value::Bool(_) => Err(ZvarError::runtime("Expected integer, found boolean")),
        }
    }

    /// Get string value, return error if not a string
    pub fn as_str(&self) -> ZvarResult<&str> {
        match self {
            Value::Str(s) => Ok(s),
            Value::Int(_) => Err(ZvarError::runtime("Expected string, found integer")),
            Value::Bool(_) => Err(ZvarError::runtime("Expected string, found boolean")),
        }
    }

    /// Get boolean value, return error if not a boolean
    pub fn as_bool(&self) -> ZvarResult<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            Value::Int(_) => Err(ZvarError::runtime("Expected boolean, found integer")),
            Value::Str(_) => Err(ZvarError::runtime("Expected boolean, found string")),
        }
    }

    /// Get integer value, panic if not an integer (for internal use)
    pub fn unwrap_int(&self) -> i64 {
        match self {
            Value::Int(n) => *n,
            Value::Str(_) => panic!("Expected integer, found string"),
            Value::Bool(_) => panic!("Expected integer, found boolean"),
        }
    }

    /// Get string value, panic if not a string (for internal use)
    pub fn unwrap_str(&self) -> &str {
        match self {
            Value::Str(s) => s,
            Value::Int(_) => panic!("Expected string, found integer"),
            Value::Bool(_) => panic!("Expected string, found boolean"),
        }
    }

    /// Get boolean value, panic if not a boolean (for internal use)
    pub fn unwrap_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(_) => panic!("Expected boolean, found integer"),
            Value::Str(_) => panic!("Expected boolean, found string"),
        }
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => *n != 0,
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
        }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
        }
    }

    /// Perform addition with another value
    pub fn add(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_add(*b)
                .map(Value::Int)
                .ok_or_else(|| ZvarError::runtime("Integer overflow")),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
            _ => Err(ZvarError::runtime(format!(
                "Cannot add {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform subtraction with another value
    pub fn sub(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_sub(*b)
                .map(Value::Int)
                .ok_or_else(|| ZvarError::runtime("Integer overflow")),
            _ => Err(ZvarError::runtime(format!(
                "Cannot subtract {} from {}",
                other.type_name(),
                self.type_name()
            ))),
        }
    }

    /// Perform multiplication with another value
    pub fn mul(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_mul(*b)
                .map(Value::Int)
                .ok_or_else(|| ZvarError::runtime("Integer overflow")),
            _ => Err(ZvarError::runtime(format!(
                "Cannot multiply {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform division with another value
    pub fn div(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(ZvarError::DivisionByZero { span: None });
                }
                a.checked_div(*b)
                    .map(Value::Int)
                    .ok_or_else(|| ZvarError::runtime("Integer overflow"))
            }
            _ => Err(ZvarError::runtime(format!(
                "Cannot divide {} by {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform equality comparison
    pub fn equal(&self, other: &Value) -> ZvarResult<Value> {
        let result = match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false, // Different types are never equal
        };
        Ok(Value::Bool(result))
    }

    /// Perform inequality comparison
    pub fn not_equal(&self, other: &Value) -> ZvarResult<Value> {
        let equal_result = self.equal(other)?;
        Ok(Value::Bool(!equal_result.unwrap_bool()))
    }

    /// Perform less-than comparison
    pub fn less(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a < b)),
            _ => Err(ZvarError::runtime(format!(
                "Cannot compare {} < {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform greater-than comparison
    pub fn greater(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a > b)),
            _ => Err(ZvarError::runtime(format!(
                "Cannot compare {} > {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform less-than-or-equal comparison
    pub fn less_equal(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a <= b)),
            _ => Err(ZvarError::runtime(format!(
                "Cannot compare {} <= {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform greater-than-or-equal comparison
    pub fn greater_equal(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Bool(a >= b)),
            _ => Err(ZvarError::runtime(format!(
                "Cannot compare {} >= {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform logical AND
    pub fn logical_and(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            _ => Err(ZvarError::runtime(format!(
                "Logical AND requires booleans, found {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform logical OR
    pub fn logical_or(&self, other: &Value) -> ZvarResult<Value> {
        match (self, other) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
            _ => Err(ZvarError::runtime(format!(
                "Logical OR requires booleans, found {} and {}",
                self.type_name(),
                other.type_name()
            ))),
        }
    }

    /// Perform logical NOT
    pub fn logical_not(&self) -> ZvarResult<Value> {
        match self {
            Value::Bool(b) => Ok(Value::Bool(!*b)),
            _ => Err(ZvarError::runtime(format!(
                "Logical NOT requires boolean, found {}",
                self.type_name()
            ))),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<crate::codegen::instruction::Value> for Value {
    fn from(val: crate::codegen::instruction::Value) -> Self {
        match val {
            crate::codegen::instruction::Value::Int(n) => Value::Int(n),
            crate::codegen::instruction::Value::Str(s) => Value::Str(s),
            crate::codegen::instruction::Value::Bool(b) => Value::Bool(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_operations() {
        let a = Value::Int(10);
        let b = Value::Int(5);

        assert_eq!(a.add(&b).unwrap(), Value::Int(15));
        assert_eq!(a.sub(&b).unwrap(), Value::Int(5));
        assert_eq!(a.mul(&b).unwrap(), Value::Int(50));
        assert_eq!(a.div(&b).unwrap(), Value::Int(2));
    }

    #[test]
    fn test_boolean_operations() {
        let true_val = Value::Bool(true);
        let false_val = Value::Bool(false);

        assert_eq!(
            true_val.logical_and(&false_val).unwrap(),
            Value::Bool(false)
        );
        assert_eq!(true_val.logical_or(&false_val).unwrap(), Value::Bool(true));
        assert_eq!(true_val.logical_not().unwrap(), Value::Bool(false));
        assert_eq!(false_val.logical_not().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_comparison_operations() {
        let a = Value::Int(10);
        let b = Value::Int(5);

        assert_eq!(a.equal(&b).unwrap(), Value::Bool(false));
        assert_eq!(a.not_equal(&b).unwrap(), Value::Bool(true));
        assert_eq!(a.greater(&b).unwrap(), Value::Bool(true));
        assert_eq!(a.less(&b).unwrap(), Value::Bool(false));
        assert_eq!(a.greater_equal(&b).unwrap(), Value::Bool(true));
        assert_eq!(b.less_equal(&a).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_operations() {
        let a = Value::Str("hello".to_string());
        let b = Value::Str(" world".to_string());

        assert_eq!(a.add(&b).unwrap(), Value::Str("hello world".to_string()));
        assert_eq!(a.equal(&b).unwrap(), Value::Bool(false));
        assert_eq!(
            a.equal(&Value::Str("hello".to_string())).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let a = Value::Int(10);
        let b = Value::Int(0);

        let result = a.div(&b);
        assert!(matches!(result, Err(ZvarError::DivisionByZero { .. })));
    }

    #[test]
    fn test_truthiness() {
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Int(-1).is_truthy());
        assert!(!Value::Int(0).is_truthy());

        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());

        assert!(Value::Str("hello".to_string()).is_truthy());
        assert!(!Value::Str("".to_string()).is_truthy());
    }

    #[test]
    fn test_type_checking() {
        let int_val = Value::Int(42);
        let str_val = Value::Str("hello".to_string());
        let bool_val = Value::Bool(true);

        assert_eq!(int_val.type_name(), "int");
        assert_eq!(str_val.type_name(), "str");
        assert_eq!(bool_val.type_name(), "bool");

        assert_eq!(int_val.unwrap_int(), 42);
        assert_eq!(str_val.unwrap_str(), "hello");
        assert_eq!(bool_val.unwrap_bool(), true);
    }

    #[test]
    fn test_conversions() {
        let int_val: Value = 42.into();
        assert_eq!(int_val, Value::Int(42));

        let bool_val: Value = true.into();
        assert_eq!(bool_val, Value::Bool(true));

        let str_val: Value = "hello".into();
        assert_eq!(str_val, Value::Str("hello".to_string()));

        let string_val: Value = "world".to_string().into();
        assert_eq!(string_val, Value::Str("world".to_string()));
    }

    #[test]
    fn test_type_errors() {
        let int_val = Value::Int(42);
        let str_val = Value::Str("hello".to_string());

        // Test arithmetic type errors
        let result = int_val.add(&str_val);
        assert!(matches!(result, Err(ZvarError::RuntimeError { .. })));

        // Test logical type errors
        let result = int_val.logical_and(&str_val);
        assert!(matches!(result, Err(ZvarError::RuntimeError { .. })));

        // Test comparison type errors
        let result = int_val.less(&str_val);
        assert!(matches!(result, Err(ZvarError::RuntimeError { .. })));
    }
}

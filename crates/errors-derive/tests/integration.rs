//! Integration tests for the SerializableError derive macro.

use errors_derive::SerializableError;
use serde_json::{from_str, Value};
use thiserror::Error;

#[derive(Debug, Error, SerializableError)]
enum UnitVariantError {
    #[error("Something went wrong")]
    SomethingWrong,
    #[error("Another error occurred")]
    AnotherError,
}

#[test]
fn test_unit_variant_serialization() {
    let err = UnitVariantError::SomethingWrong;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "somethingWrong");
    assert_eq!(parsed["message"], "Something went wrong");
}

#[test]
fn test_unit_variant_another() {
    let err = UnitVariantError::AnotherError;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "anotherError");
    assert_eq!(parsed["message"], "Another error occurred");
}

#[derive(Debug, Error, SerializableError)]
enum TupleVariantError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[test]
fn test_tuple_variant_serialization() {
    let err = TupleVariantError::Parse("invalid syntax".to_string());
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "parse");
    assert_eq!(parsed["message"], "Parse error: invalid syntax");
}

#[test]
fn test_tuple_variant_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = TupleVariantError::Io(io_err);
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "io");
    assert!(parsed["message"].as_str().is_some_and(|s| s.contains("file not found")));
}

#[derive(Debug, Error, SerializableError)]
enum NamedFieldError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    #[error("Invalid config: {key} = {value}")]
    InvalidConfig { key: String, value: String },
}

#[test]
fn test_named_field_single() {
    let err = NamedFieldError::FileNotFound {
        path: "/etc/config".to_string(),
    };
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "fileNotFound");
    assert_eq!(parsed["message"], "File not found: /etc/config");
}

#[test]
fn test_named_field_multiple() {
    let err = NamedFieldError::InvalidConfig {
        key: "timeout".to_string(),
        value: "-1".to_string(),
    };
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "invalidConfig");
    assert_eq!(parsed["message"], "Invalid config: timeout = -1");
}

#[derive(Debug, Error, SerializableError)]
enum CustomCodeError {
    #[error("File not found")]
    #[error_code("FILE_NOT_FOUND")]
    FileNotFound,
    #[error("Permission denied")]
    #[error_code("PERMISSION_DENIED")]
    PermissionDenied,
    #[error("Unknown error")]
    Unknown,
}

#[test]
fn test_custom_error_code() {
    let err = CustomCodeError::FileNotFound;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "FILE_NOT_FOUND");
    assert_eq!(parsed["message"], "File not found");
}

#[test]
fn test_custom_error_code_permission() {
    let err = CustomCodeError::PermissionDenied;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "PERMISSION_DENIED");
    assert_eq!(parsed["message"], "Permission denied");
}

#[test]
fn test_default_camel_case_without_custom_code() {
    let err = CustomCodeError::Unknown;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "unknown");
    assert_eq!(parsed["message"], "Unknown error");
}

#[derive(Debug, Error, SerializableError)]
enum MixedError {
    #[error("Unit variant")]
    Unit,
    #[error("Tuple with one: {0}")]
    TupleOne(i32),
    #[error("Tuple with two: {0}, {1}")]
    TupleTwo(String, i32),
    #[error("Named: {name} at {line}")]
    Named { name: String, line: u32 },
}

#[test]
fn test_mixed_unit() {
    let err = MixedError::Unit;
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "unit");
    assert_eq!(parsed["message"], "Unit variant");
}

#[test]
fn test_mixed_tuple_one() {
    let err = MixedError::TupleOne(42);
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "tupleOne");
    assert_eq!(parsed["message"], "Tuple with one: 42");
}

#[test]
fn test_mixed_tuple_two() {
    let err = MixedError::TupleTwo("test".to_string(), 123);
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "tupleTwo");
    assert_eq!(parsed["message"], "Tuple with two: test, 123");
}

#[test]
fn test_mixed_named() {
    let err = MixedError::Named {
        name: "main".to_string(),
        line: 42,
    };
    let json = serde_json::to_string(&err).expect("Failed to serialize");
    let parsed: Value = from_str(&json).expect("Failed to parse JSON");

    assert_eq!(parsed["kind"], "named");
    assert_eq!(parsed["message"], "Named: main at 42");
}
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Display, Formatter},
    vec::Vec,
};

/// DataCollection is returned when a find or search returns
/// multiple Data objects
#[derive(Serialize, Deserialize, Debug)]
pub struct DataCollection {
    pub data: Vec<Data>,
}

/// `Data` stores a unit of data in the redact system. A chunk of
/// data is a `DataValue` (contained within), which can be a `bool`,
/// `u64`, `i64`, `f64`, or `string`. Each data is associated with a `DataPath`
/// which is just a json-style path, and can optionally be encrypted
/// by a variety of keys as specified by the key names in `encryptedby`.
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Data {
    path: DataPath,
    #[serde(default)]
    value: DataValueCollection,
    encryptedby: Option<Vec<String>>,
}

impl Data {
    /// Builds a new Data struct using the provided values
    pub fn new(path: &str, value: DataValue, encryptedby: Option<Vec<String>>) -> Self {
        Data {
            path: DataPath::from(path),
            value: DataValueCollection(vec![value]),
            encryptedby,
        }
    }

    /// Returns an owned string representing the data's jsonpath
    pub fn path(&self) -> String {
        self.path.to_string()
    }

    /// Returns the optional list of keys this data is encrypted by
    pub fn encryptedby(&self) -> &Option<Vec<String>> {
        &self.encryptedby
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value.to_string())
    }
}

/// Wraps a vector of `DataValue` enums. In the future, this type will implement
/// group `DataValue` operations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DataValueCollection(Vec<DataValue>);

impl Display for DataValueCollection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|dv| write!(f, "{}", dv.to_string()))
    }
}

/// `DataValue` contains the actual raw value of a piece of `Data`.
/// A `DataValue` should always be a leaf value, not an array or object.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DataValue {
    Encrypted(EncryptedDataValue),
    Unencrypted(UnencryptedDataValue),
}

impl Default for DataValue {
    fn default() -> Self {
        Self::Unencrypted(UnencryptedDataValue::Bool(false))
    }
}

impl Display for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DataValue::Encrypted(ref e) => write!(f, "{}", e.to_string()),
            DataValue::Unencrypted(ref u) => write!(f, "{}", u.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Bool,
    U64,
    I64,
    F64,
    String,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Bool => write!(f, "bool"),
            Self::U64 => write!(f, "u64"),
            Self::I64 => write!(f, "i64"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UnencryptedDataValue {
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    String(String),
}

impl Display for UnencryptedDataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            UnencryptedDataValue::Bool(ref b) => write!(f, "{}", b),
            UnencryptedDataValue::U64(ref n) => write!(f, "{}", n),
            UnencryptedDataValue::I64(ref n) => write!(f, "{}", n),
            UnencryptedDataValue::F64(ref n) => write!(f, "{}", n),
            UnencryptedDataValue::String(ref s) => write!(f, "{}", s),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EncryptedDataValue {
    value: Vec<u8>,
    datatype: DataType,
    keyname: String,
}

impl Display for EncryptedDataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "encrypted(key: \"{}\", type: \"{}\", value: \"{}\")",
            self.keyname,
            self.datatype,
            String::from_utf8(self.value.clone()).map_err(|_| std::fmt::Error)?,
        )
    }
}

// impl From<DataValue> for String {
//     fn from(val: DataValue) -> Self {
//         val.to_string()
//     }
// }

impl From<String> for DataValue {
    fn from(s: String) -> Self {
        DataValue::Unencrypted(UnencryptedDataValue::String(s))
    }
}

impl From<&str> for DataValue {
    fn from(s: &str) -> Self {
        DataValue::Unencrypted(UnencryptedDataValue::String(s.to_owned()))
    }
}

// impl From<Value> for DataValue {
//     fn from(v: Value) -> Self {
//         match v {
//             Value::Null => DataValue::String("".to_owned()),
//             Value::Bool(b) => DataValue::Bool(b),
//             Value::Number(n) => DataValue::from(n.to_string().as_ref()),
//             Value::String(s) => DataValue::String(s),
//             _ => DataValue::String(v.to_string()),
//         }
//     }
// }

/// `DataPath` represents a json-style path for the location of a `Data` object.
/// The path should always be formatted as `.my.json.path.`; note the beginning and
/// ending periods. `DataPath` will automatically handle path validation when
/// created or deserialized, just provide any valid json-path on creation.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(into = "String", from = "String")]
pub struct DataPath {
    path: String,
}

impl DataPath {
    /// Validates a given string and returns a new DataPath
    pub fn new(path: &str) -> Self {
        let path = Self::validate_path(path);
        Self { path }
    }

    // Ensures that a data entry path begins and ends with a period ('.')
    // Empty strings will return as "."
    // Strings of length 1 where the only char is a period will return as "."
    // All other strings will have periods added to the beginning or end if needed.
    // For now, string containing multiple periods in a row, or composed only of
    // multiple periods, will be accepted and returned as given, with the same
    // behavior as any other standard string of len > 1.
    // This function is implemented as a boolean circuit to avoid iterating through
    // the same string numerous times.
    fn validate_path(path: &str) -> String {
        // Short circuit if path is empty
        if path.is_empty() {
            return ".".to_owned();
        }

        // Collect the first and last characters of the path
        let mut path_chars = path.chars();
        let first_char = path_chars.next();
        let last_char = path_chars.last();

        // Match on the results of char extraction
        match (first_char, last_char) {
            // String length >= 2
            (Some(fc), Some(lc)) => {
                if fc != '.' && lc != '.' {
                    format!(".{}.", path)
                } else if fc == '.' && lc == '.' {
                    path.to_owned()
                } else if fc != '.' {
                    format!(".{}", path)
                } else {
                    format!("{}.", path)
                }
            }
            // String length == 1
            (Some(fc), None) => {
                if fc == '.' {
                    path.to_owned()
                } else {
                    format!(".{}.", path)
                }
            }
            // Impossible case: string length == 0, should never be here because
            // of the short-circuit implemented at the beginning of the function
            (None, None) => panic!(
                "this is an impossible situation; if you have gotten here, \\
	     a short-circuit earlier in the function has failed to function as \\
	     intended"
            ),
            // Impossible case: if this happens we should panic because something is
            // fundamentally wrong with the computing environment and someone should
            // know about it.
            // If the last char is != None, then it MUST BE that the
            // first char is != None, as the last char is collected after the
            // iterator has ticked over one spot to account for the first char,
            // therefore if the iterator finds something in the last() call, then
            // it must be after having collected something from the nth(0) call.
            (None, Some(_)) => panic!(
                "this is an impossible situation; if you have gotten here, \\
	     something has happened that should never happen according to the \\
	     laws of computing and/or the rust compiler. if you have gotten here, \\
	     some major memory or computing trickery has occurred and you should \\
	     be concerned for the integrity of your computing base"
            ),
        }
    }
}

impl Display for DataPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl<'a> From<&'a str> for DataPath {
    fn from(path: &'a str) -> Self {
        Self::new(path)
    }
}

/// We need this because of a requirement on Deserialize
/// Prefer not to use it, using the &str version instead
impl From<String> for DataPath {
    fn from(path: String) -> Self {
        Self::from(path.as_ref())
    }
}

impl From<DataPath> for String {
    fn from(dp: DataPath) -> Self {
        dp.to_string()
    }
}

#[cfg(test)]
mod tests {
    mod datavaluecollection {
        use crate::data::DataValueCollection;

        #[test]
        fn test_default_is_empty_vec() {
            let dvc = DataValueCollection::default();
            assert!(dvc.0.is_empty());
        }

        #[test]
        fn test_to_string_empty() {
            let dvc = DataValueCollection::default();
            assert!(dvc.0.is_empty());
            assert_eq!("", dvc.to_string());
        }
    }
    mod datavalue {
        use crate::data::{DataType, DataValue, EncryptedDataValue, UnencryptedDataValue};

        #[test]
        fn test_to_string_encrypted() {
            let dv = DataValue::Encrypted(EncryptedDataValue {
                value: "hello".into(),
                datatype: DataType::String,
                keyname: "somekey".to_owned(),
            });

            assert_eq!(
                "encrypted(key: \"somekey\", type: \"string\", value: \"hello\")",
                dv.to_string()
            )
        }

        #[test]
        fn test_to_string_unencrypted() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::Bool(true));

            assert_eq!("true", dv.to_string())
        }
    }
    mod datatype {
        use crate::data::DataType;

        #[test]
        fn test_to_string_bool() {
            let dt = DataType::Bool;

            assert_eq!("bool", dt.to_string())
        }

        #[test]
        fn test_to_string_u64() {
            let dt = DataType::U64;

            assert_eq!("u64", dt.to_string())
        }

        #[test]
        fn test_to_string_i64() {
            let dt = DataType::I64;

            assert_eq!("i64", dt.to_string())
        }

        #[test]
        fn test_to_string_f64() {
            let dt = DataType::F64;

            assert_eq!("f64", dt.to_string())
        }

        #[test]
        fn test_to_string_string() {
            let dt = DataType::String;

            assert_eq!("string", dt.to_string())
        }
    }
    mod unencrypteddatavalue {
        use crate::data::{DataValue, UnencryptedDataValue};

        #[test]
        fn test_to_string_unencrypted_bool_true() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::Bool(true));

            assert_eq!("true", dv.to_string())
        }

        #[test]
        fn test_to_string_unencrypted_bool_false() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::Bool(false));

            assert_eq!("false", dv.to_string())
        }

        #[test]
        fn test_to_string_unencrypted_u64() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::U64(0));

            assert_eq!("0", dv.to_string())
        }

        #[test]
        fn test_to_string_unencrypted_i64() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::I64(-10));

            assert_eq!("-10", dv.to_string())
        }

        #[test]
        fn test_to_string_unencrypted_f64() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::F64(-390.321));

            assert_eq!("-390.321", dv.to_string())
        }

        #[test]
        fn test_to_string_unencrypted_string() {
            let dv = DataValue::Unencrypted(UnencryptedDataValue::String("hello".to_owned()));

            assert_eq!("hello", dv.to_string())
        }
    }
    mod encrypteddatavalue {
        use crate::data::{DataType, DataValue, EncryptedDataValue};

        #[test]
        fn test_to_string_encrypted() {
            let dv = DataValue::Encrypted(EncryptedDataValue {
                value: "hello".into(),
                datatype: DataType::String,
                keyname: "somekey".to_owned(),
            });

            assert_eq!(
                "encrypted(key: \"somekey\", type: \"string\", value: \"hello\")",
                dv.to_string()
            )
        }
    }

    // #[test]
    // fn test_default_is_false_bool() {
    //     let dv = DataValue::default();
    //     match dv {
    //         DataValue::Unencrypted(e) => {
    // 	    match e {
    // 		UnencryptedDataValue::Bool(b) => assert!(!b,  "default DataValue should be a DataValue::Bool(false)"),
    // 		_ => panic!("default DataValue is an unencrypted non-bool, should be an unencrypted bool with value false"),
    // 	    }
    // 	}
    //         _ => {
    //             panic!("default DataValue should be a DataValue::Bool(false)")
    //         }
    //     }
    // }

    //     #[test]
    //     fn test_to_string_bool_true() {
    //         let dv = DataValue::from("true");
    //         assert_eq!(dv.to_string(), "true");
    //     }

    //     #[test]
    //     fn test_to_string_bool_false() {
    //         let dv = DataValue::from("false");
    //         assert_eq!(dv.to_string(), "false");
    //     }

    //     #[test]
    //     fn test_to_string_u64() {
    //         let dv = DataValue::from("24");
    //         assert_eq!(dv.to_string(), "24");
    //     }

    //     #[test]
    //     fn test_to_string_i64() {
    //         let dv = DataValue::from("-10");
    //         assert_eq!(dv.to_string(), "-10");
    //     }

    //     #[test]
    //     fn test_to_string_f64() {
    //         let dv = DataValue::from("10.3");
    //         assert_eq!(dv.to_string(), "10.3");
    //     }

    //     #[test]
    //     fn test_to_string_string() {
    //         let dv = DataValue::from("somestr");
    //         assert_eq!(dv.to_string(), "somestr");
    //     }

    //     #[test]
    //     fn test_from_datavalue_for_string() {
    //         let dv = DataValue::default();
    //         let s: String = From::<DataValue>::from(dv);
    //         assert_eq!(s, "false");
    //     }

    //     #[test]
    //     fn test_from_string_for_bool_true() {
    //         let dv = From::<&str>::from("true");
    //         match dv {
    //             DataValue::Bool(b) => assert!(b),
    //             _ => panic!("DataValue should have been a Bool variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_bool_false() {
    //         let dv = From::<&str>::from("false");
    //         match dv {
    //             DataValue::Bool(b) => assert!(!b),
    //             _ => panic!("DataValue should have been a Bool variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_zero() {
    //         let dv = From::<&str>::from("0");
    //         match dv {
    //             DataValue::U64(n) => assert_eq!(n, 0),
    //             _ => panic!("DataValue should have been a U64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_positive_integer() {
    //         let dv = From::<&str>::from("100");
    //         match dv {
    //             DataValue::U64(n) => assert_eq!(n, 100),
    //             _ => panic!("DataValue should have been a U64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_negative_integer() {
    //         let dv = From::<&str>::from("-1");
    //         match dv {
    //             DataValue::I64(n) => assert_eq!(n, -1),
    //             _ => panic!("DataValue should have been a I64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_positive_decimal() {
    //         let dv = From::<&str>::from("10.52");
    //         match dv {
    //             // We have to do the f64::EPSILON comparison here as floating point
    //             // comparisons are inherently inexact; see:
    //             // https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp
    //             DataValue::F64(n) => assert!((n - 10.52f64).abs() < f64::EPSILON),
    //             _ => panic!("DataValue should have been a F64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_negative_decimal() {
    //         let dv = From::<&str>::from("-4.38");
    //         match dv {
    //             // We have to do the f64::EPSILON comparison here as floating point
    //             // comparisons are inherently inexact; see:
    //             // https://rust-lang.github.io/rust-clippy/master/index.html#float_cmp
    //             DataValue::F64(n) => assert!((n + 4.38f64).abs() < f64::EPSILON),
    //             _ => panic!("DataValue should have been a F64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_string() {
    //         let dv = From::<&str>::from("somestr");
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, "somestr"),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_string_that_starts_with_a_number() {
    //         let dv = From::<&str>::from("10.52a");
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, "10.52a"),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_string_for_empty_string() {
    //         let dv = From::<&str>::from("");
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, ""),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_null_variant() {
    //         let dv = From::<Value>::from(Value::Null);
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, ""),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_bool_true_variant() {
    //         let dv = From::<Value>::from(Value::Bool(true));
    //         match dv {
    //             DataValue::Bool(b) => assert!(b),
    //             _ => panic!("DataValue should have been a Bool variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_bool_false_variant() {
    //         let dv = From::<Value>::from(Value::Bool(false));
    //         match dv {
    //             DataValue::Bool(b) => assert!(!b),
    //             _ => panic!("DataValue should have been a Bool variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_number_zero_variant() {
    //         let dv = From::<Value>::from(json!(0));
    //         match dv {
    //             DataValue::U64(n) => assert_eq!(n, 0),
    //             _ => panic!("DataValue should have been a U64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_number_negative_variant() {
    //         let dv = From::<Value>::from(json!(-1240));
    //         match dv {
    //             DataValue::I64(n) => assert_eq!(n, -1240),
    //             _ => panic!("DataValue should have been an I64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_number_negative_decimal_variant() {
    //         let dv = From::<Value>::from(json!(-300.434));
    //         match dv {
    //             DataValue::F64(n) => assert!((n + 300.434).abs() < f64::EPSILON),
    //             _ => panic!("DataValue should have been an F64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_number_positive_decimal_variant() {
    //         let dv = From::<Value>::from(json!(0.001));
    //         match dv {
    //             DataValue::F64(n) => assert!((n - 0.001).abs() < f64::EPSILON),
    //             _ => panic!("DataValue should have been an F64 variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_string_variant() {
    //         let dv = From::<Value>::from(Value::String("somestr".to_owned()));
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, "somestr"),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_object_variant() {
    //         let dv = From::<Value>::from(json!({ "key": "value" }));
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, "{\"key\":\"value\"}"),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }

    //     #[test]
    //     fn test_from_value_for_array_variant() {
    //         let dv = From::<Value>::from(json!([ 1, "str", { "key": "value" } ]));
    //         match dv {
    //             DataValue::String(s) => assert_eq!(s, "[1,\"str\",{\"key\":\"value\"}]"),
    //             _ => panic!("DataValue should have been a String variant"),
    //         }
    //     }
    // }

    mod datapath {
        use crate::data::DataPath;
        use std::convert::From;

        #[test]
        fn test_new_with_valid_path() {
            let dp = DataPath::new(".my.path.");
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_new_with_path_missing_first_period() {
            let dp = DataPath::new("my.path.");
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_new_with_path_missing_last_period() {
            let dp = DataPath::new(".my.path");
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_new_with_path_missing_first_and_last_period() {
            let dp = DataPath::new("my.path");
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_new_with_path_with_no_periods() {
            let dp = DataPath::new("my");
            assert_eq!(dp.to_string(), ".my.");
        }

        #[test]
        fn test_new_with_empty_path() {
            let dp = DataPath::new("");
            assert_eq!(dp.to_string(), ".");
        }

        #[test]
        fn test_new_with_path_is_single_period() {
            let dp = DataPath::new(".");
            assert_eq!(dp.to_string(), ".");
        }

        #[test]
        fn test_new_with_path_is_double_period() {
            let dp = DataPath::new("..");
            assert_eq!(dp.to_string(), "..");
        }

        #[test]
        fn test_from_string() {
            let dp: DataPath = From::<String>::from("my.path".to_owned());
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_from_str() {
            let dp: DataPath = From::<&str>::from("my.path.");
            assert_eq!(dp.to_string(), ".my.path.");
        }

        #[test]
        fn test_from_datapath_for_string() {
            let dp = DataPath::new(".my.path");
            let s: String = From::<DataPath>::from(dp);
            assert_eq!(s, ".my.path.");
        }
    }
}

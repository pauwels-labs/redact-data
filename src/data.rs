use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataCollection {
    pub data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Data {
    #[serde(
        serialize_with = "DataPath::serialize_datapath",
        deserialize_with = "DataPath::deserialize_datapath"
    )]
    pub path: DataPath,
    #[serde(default)]
    pub value: DataValue,
    pub encryptedby: Option<Vec<String>>,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(into = "String", from = "String")]
pub enum DataValue {
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    String(String),
}

impl Default for DataValue {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl From<DataValue> for String {
    fn from(val: DataValue) -> Self {
        val.to_string()
    }
}

impl From<String> for DataValue {
    fn from(s: String) -> Self {
        if let Ok(b) = s.parse::<bool>() {
            DataValue::Bool(b)
        } else if let Ok(n) = s.parse::<u64>() {
            DataValue::U64(n)
        } else if let Ok(n) = s.parse::<i64>() {
            DataValue::I64(n)
        } else if let Ok(n) = s.parse::<f64>() {
            DataValue::F64(n)
        } else {
            DataValue::String(s)
        }
    }
}

impl Display for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DataValue::Bool(ref b) => write!(f, "{}", b),
            DataValue::U64(ref n) => write!(f, "{}", n),
            DataValue::I64(ref n) => write!(f, "{}", n),
            DataValue::F64(ref n) => write!(f, "{}", n),
            DataValue::String(ref s) => write!(f, "{}", s),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DataPath {
    #[serde(deserialize_with = "DataPath::deserialize_path")]
    path: String,
}

impl DataPath {
    pub fn new(path: &str) -> Self {
        let path = Self::validate_path(path);
        Self { path }
    }

    fn serialize_datapath<S>(dp: &Self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(&dp.to_string())
    }

    fn deserialize_datapath<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::new(&String::deserialize(deserializer)?))
    }

    fn deserialize_path<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::validate_path(&String::deserialize(deserializer)?))
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

impl From<String> for DataPath {
    fn from(path: String) -> Self {
        Self::new(&path)
    }
}

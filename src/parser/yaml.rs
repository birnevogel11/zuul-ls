use std::path::Path;
use std::{collections::BTreeMap, mem};

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser, Tag};
use yaml_rust2::scanner::{Marker, ScanError, TScalarStyle};

use hashlink::LinkedHashMap;

/// The type contained in the `Yaml::Array` variant. This corresponds to YAML sequences.
pub type Array = Vec<YValue>;
/// The type contained in the `Yaml::Hash` variant. This corresponds to YAML mappings.
pub type Hash = LinkedHashMap<YValue, YValue>;

// parse f64 as Core schema
// See: https://github.com/chyh1990/yaml-rust/issues/51
fn parse_f64(v: &str) -> Option<f64> {
    match v {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        ".nan" | "NaN" | ".NAN" => Some(f64::NAN),
        _ => v.parse::<f64>().ok(),
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct Loc {
    line: usize,
    col: usize,
}

impl Loc {
    pub fn new(mark: &Marker) -> Loc {
        Loc {
            line: mark.line(),
            col: mark.col(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct YValue {
    value: YValueYaml,
    loc: Loc,
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum YValueYaml {
    /// Float types are stored as String and parsed on demand.
    /// Note that `f64` does NOT implement Eq trait and can NOT be stored in `BTreeMap`.
    Real(String),
    /// YAML int is stored as i64.
    Integer(i64),
    /// YAML scalar.
    String(String),
    /// YAML bool, e.g. `true` or `false`.
    Boolean(bool),
    /// YAML array, can be accessed as a `Vec`.
    Array(Array),
    /// YAML hash, can be accessed as a `LinkedHashMap`.
    ///
    /// Insertion order will match the order of insertion into the map.
    Hash(Hash),
    /// Alias, not fully supported yet.
    Alias(usize),
    /// YAML null, e.g. `null` or `~`.
    Null,
    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    BadValue,
}

macro_rules! define_as (
    ($name:ident, $t:ident, $yt:ident) => (
/// Get a copy of the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some($t)` with a copy of the `$t` contained.
/// Otherwise, return `None`.
#[must_use]
pub fn $name(&self) -> Option<$t> {
    match self.value {
        YValueYaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_ref (
    ($name:ident, $t:ty, $yt:ident) => (
/// Get a reference to the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some(&$t)` with the `$t` contained. Otherwise,
/// return `None`.
#[must_use]
pub fn $name(&self) -> Option<$t> {
    match self.value {
        YValueYaml::$yt(ref v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_mut_ref (
    ($name:ident, $t:ty, $yt:ident) => (
/// Get a mutable reference to the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some(&mut $t)` with the `$t` contained.
/// Otherwise, return `None`.
#[must_use]
pub fn $name(&mut self) -> Option<$t> {
    match self.value {
        YValueYaml::$yt(ref mut v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_into (
    ($name:ident, $t:ty, $yt:ident) => (
/// Get the inner object in the YAML enum if it is a `$t`.
///
/// # Return
/// If the variant of `self` is `Yaml::$yt`, return `Some($t)` with the `$t` contained. Otherwise,
/// return `None`.
#[must_use]
pub fn $name(self) -> Option<$t> {
    match self.value {
        YValueYaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

impl YValue {
    define_as!(as_bool, bool, Boolean);
    define_as!(as_i64, i64, Integer);

    define_as_ref!(as_str, &str, String);
    define_as_ref!(as_hash, &Hash, Hash);
    define_as_ref!(as_vec, &Array, Array);

    define_as_mut_ref!(as_mut_hash, &mut Hash, Hash);
    define_as_mut_ref!(as_mut_vec, &mut Array, Array);

    define_into!(into_bool, bool, Boolean);
    define_into!(into_i64, i64, Integer);
    define_into!(into_string, String, String);
    define_into!(into_hash, Hash, Hash);
    define_into!(into_vec, Array, Array);

    pub fn new(value: YValueYaml, mark: &Marker) -> YValue {
        YValue {
            value,
            loc: Loc::new(mark),
        }
    }

    pub fn value(&self) -> &YValueYaml {
        &self.value
    }

    pub fn loc(&self) -> &Loc {
        &self.loc
    }

    pub fn line(&self) -> usize {
        self.loc.line
    }

    pub fn col(&self) -> usize {
        self.loc.col
    }

    /// Return whether `self` is a [`Yaml::BadValue`] node.
    #[must_use]
    pub fn is_badvalue(&self) -> bool {
        matches!(self.value, YValueYaml::BadValue)
    }

    #[must_use]
    pub fn from_str(v: &str, mark: &Marker) -> YValue {
        if let Some(number) = v.strip_prefix("0x") {
            if let Ok(i) = i64::from_str_radix(number, 16) {
                return YValue::new(YValueYaml::Integer(i), mark);
            }
        } else if let Some(number) = v.strip_prefix("0o") {
            if let Ok(i) = i64::from_str_radix(number, 8) {
                return YValue::new(YValueYaml::Integer(i), mark);
            }
        } else if let Some(number) = v.strip_prefix('+') {
            if let Ok(i) = number.parse::<i64>() {
                return YValue::new(YValueYaml::Integer(i), mark);
            }
        }
        match v {
            "~" | "null" => YValue::new(YValueYaml::Null, mark),
            "true" => YValue::new(YValueYaml::Boolean(true), mark),
            "false" => YValue::new(YValueYaml::Boolean(false), mark),
            _ => {
                if let Ok(integer) = v.parse::<i64>() {
                    YValue::new(YValueYaml::Integer(integer), mark)
                } else if parse_f64(v).is_some() {
                    YValue::new(YValueYaml::Real(v.to_owned()), mark)
                } else {
                    YValue::new(YValueYaml::String(v.to_owned()), mark)
                }
            }
        }
    }
}

/// Main structure for quickly parsing YAML.
///
/// See [`YamlLoader::load_from_str`].
#[derive(Default)]
pub struct YValueLoader {
    /// The different YAML documents that are loaded.
    docs: Vec<YValue>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(YValue, usize)>,
    key_stack: Vec<YValue>,
    anchor_map: BTreeMap<usize, YValue>,
    /// An error, if one was encountered.
    error: Option<ScanError>,
}

impl MarkedEventReceiver for YValueLoader {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        if self.error.is_some() {
            return;
        }
        if let Err(e) = self.on_event_impl(ev, mark) {
            self.error = Some(e);
        }
    }
}

impl YValueLoader {
    fn on_event_impl(&mut self, ev: Event, mark: Marker) -> Result<(), ScanError> {
        match ev {
            Event::DocumentStart | Event::Nothing | Event::StreamStart | Event::StreamEnd => {
                // do nothing
            }
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(YValue::new(YValueYaml::BadValue, &mark)),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid, _) => {
                self.doc_stack
                    .push((YValue::new(YValueYaml::Array(Vec::new()), &mark), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node, mark)?;
            }
            Event::MappingStart(aid, _) => {
                self.doc_stack
                    .push((YValue::new(YValueYaml::Hash(Hash::new()), &mark), aid));
                self.key_stack
                    .push(YValue::new(YValueYaml::BadValue, &mark));
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node, mark)?;
            }
            Event::Scalar(v, style, aid, tag) => {
                let node = if style != TScalarStyle::Plain {
                    YValue::new(YValueYaml::String(v), &mark)
                } else if let Some(Tag {
                    ref handle,
                    ref suffix,
                }) = tag
                {
                    if handle == "tag:yaml.org,2002:" {
                        match suffix.as_ref() {
                            "bool" => {
                                // "true" or "false"
                                match v.parse::<bool>() {
                                    Err(_) => YValue::new(YValueYaml::BadValue, &mark),
                                    Ok(v) => YValue::new(YValueYaml::Boolean(v), &mark),
                                }
                            }
                            "int" => match v.parse::<i64>() {
                                Err(_) => YValue::new(YValueYaml::BadValue, &mark),
                                Ok(v) => YValue::new(YValueYaml::Integer(v), &mark),
                            },
                            "float" => match parse_f64(&v) {
                                Some(_) => YValue::new(YValueYaml::Real(v), &mark),
                                None => YValue::new(YValueYaml::BadValue, &mark),
                            },
                            "null" => match v.as_ref() {
                                "~" | "null" => YValue::new(YValueYaml::Null, &mark),
                                _ => YValue::new(YValueYaml::BadValue, &mark),
                            },
                            _ => YValue::new(YValueYaml::String(v), &mark),
                        }
                    } else {
                        YValue::new(YValueYaml::String(v), &mark)
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    YValue::from_str(&v, &mark)
                };

                self.insert_new_node((node, aid), mark)?;
            }
            Event::Alias(id) => {
                let n = match self.anchor_map.get(&id) {
                    Some(v) => v.clone(),
                    None => YValue::new(YValueYaml::BadValue, &mark),
                };
                self.insert_new_node((n, 0), mark)?;
            } // _ => unreachable!(),
        }

        Ok(())
    }

    fn insert_new_node(&mut self, node: (YValue, usize), mark: Marker) -> Result<(), ScanError> {
        // valid anchor id starts from 1
        if node.1 > 0 {
            self.anchor_map.insert(node.1, node.0.clone());
        }
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = &mut self.doc_stack.last_mut().unwrap().0;
            match parent.value {
                YValueYaml::Array(ref mut v) => v.push(node.0),
                YValueYaml::Hash(ref mut h) => {
                    let cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = YValue::new(YValueYaml::BadValue, &mark);
                        mem::swap(&mut newkey, cur_key);
                        if h.insert(newkey, node.0).is_some() {
                            let inserted_key = h.back().unwrap().0;
                            return Err(ScanError::new(
                                mark,
                                format!("{inserted_key:?}: duplicated key in mapping").as_str(),
                            ));
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    /// Load the given string as a set of YAML documents.
    ///
    /// The `source` is interpreted as YAML documents and is parsed. Parsing succeeds if and only
    /// if all documents are parsed successfully. An error in a latter document prevents the former
    /// from being returned.
    /// # Errors
    /// Returns `ScanError` when loading fails.
    pub fn load_from_str(source: &str) -> Result<Vec<YValue>, ScanError> {
        Self::load_from_iter(source.chars())
    }

    /// Load the contents of the given iterator as a set of YAML documents.
    ///
    /// The `source` is interpreted as YAML documents and is parsed. Parsing succeeds if and only
    /// if all documents are parsed successfully. An error in a latter document prevents the former
    /// from being returned.
    /// # Errors
    /// Returns `ScanError` when loading fails.
    pub fn load_from_iter<I: Iterator<Item = char>>(source: I) -> Result<Vec<YValue>, ScanError> {
        let mut parser = Parser::new(source);
        Self::load_from_parser(&mut parser)
    }

    /// Load the contents from the specified Parser as a set of YAML documents.
    ///
    /// Parsing succeeds if and only if all documents are parsed successfully.
    /// An error in a latter document prevents the former from being returned.
    /// # Errors
    /// Returns `ScanError` when loading fails.
    pub fn load_from_parser<I: Iterator<Item = char>>(
        parser: &mut Parser<I>,
    ) -> Result<Vec<YValue>, ScanError> {
        let mut loader = YValueLoader::default();
        parser.load(&mut loader, true)?;
        if let Some(e) = loader.error {
            Err(e)
        } else {
            Ok(loader.docs)
        }
    }

    /// Return a reference to the parsed Yaml documents.
    #[must_use]
    pub fn documents(&self) -> &[YValue] {
        &self.docs
    }
}

/// An error that occurred while scanning.
#[derive(Debug)]
pub enum LoadYValueError {
    FileError(std::io::Error),
    ParseError(ScanError),
}

pub fn load_yvalue(path: &Path) -> Result<Vec<YValue>, LoadYValueError> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(err) => Err(LoadYValueError::FileError(err)),
    };

    match content {
        Ok(content) => match YValueLoader::load_from_str(&content) {
            Ok(docs) => Ok(docs),
            Err(err) => Err(LoadYValueError::ParseError(err)),
        },
        Err(err) => Err(err),
    }
}

pub fn load_yvalue_from_str(content: &str) -> Result<Vec<YValue>, LoadYValueError> {
    match YValueLoader::load_from_str(content) {
        Ok(docs) => Ok(docs),
        Err(err) => Err(LoadYValueError::ParseError(err)),
    }
}

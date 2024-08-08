use std::{collections::BTreeMap, convert::TryFrom, mem, ops::Index, ops::IndexMut};

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser, Tag};
use yaml_rust2::scanner::{Marker, ScanError, TScalarStyle};
use yaml_rust2::yaml::LoadError;

use hashlink::LinkedHashMap;

/// The type contained in the `Yaml::Array` variant. This corresponds to YAML sequences.
pub type Array = Vec<YValue>;
/// The type contained in the `Yaml::Hash` variant. This corresponds to YAML mappings.
pub type Hash = LinkedHashMap<YValue, YValue>;

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct Loc {
    index: usize,
    line: usize,
    col: usize,
}

impl Loc {
    pub fn new(mark: &Marker) -> Loc {
        Loc {
            index: mark.index(),
            line: mark.line(),
            col: mark.col(),
        }
    }
}

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
pub enum YValue {
    /// Float types are stored as String and parsed on demand.
    /// Note that `f64` does NOT implement Eq trait and can NOT be stored in `BTreeMap`.
    Real(String, Loc),
    /// YAML int is stored as i64.
    Integer(i64, Loc),
    /// YAML scalar.
    String(String, Loc),
    /// YAML bool, e.g. `true` or `false`.
    Boolean(bool, Loc),
    /// YAML array, can be accessed as a `Vec`.
    Array(Array, Loc),
    /// YAML hash, can be accessed as a `LinkedHashMap`.
    ///
    /// Insertion order will match the order of insertion into the map.
    Hash(Hash, Loc),
    /// Alias, not fully supported yet.
    Alias(usize, Loc),
    /// YAML null, e.g. `null` or `~`.
    Null(Loc),
    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    BadValue(Loc),
}

impl YValue {
    /// Return whether `self` is a [`Yaml::BadValue`] node.
    #[must_use]
    pub fn is_badvalue(&self) -> bool {
        matches!(*self, YValue::BadValue(_))
    }

    #[must_use]
    pub fn from_str(v: &str, mark: &Marker) -> YValue {
        if let Some(number) = v.strip_prefix("0x") {
            if let Ok(i) = i64::from_str_radix(number, 16) {
                return YValue::Integer(i, Loc::new(&mark));
            }
        } else if let Some(number) = v.strip_prefix("0o") {
            if let Ok(i) = i64::from_str_radix(number, 8) {
                return YValue::Integer(i, Loc::new(&mark));
            }
        } else if let Some(number) = v.strip_prefix('+') {
            if let Ok(i) = number.parse::<i64>() {
                return YValue::Integer(i, Loc::new(&mark));
            }
        }
        match v {
            "~" | "null" => YValue::Null(Loc::new(&mark)),
            "true" => YValue::Boolean(true, Loc::new(&mark)),
            "false" => YValue::Boolean(false, Loc::new(&mark)),
            _ => {
                if let Ok(integer) = v.parse::<i64>() {
                    YValue::Integer(integer, Loc::new(&mark))
                } else if parse_f64(v).is_some() {
                    YValue::Real(v.to_owned(), Loc::new(&mark))
                } else {
                    YValue::String(v.to_owned(), Loc::new(&mark))
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
                    0 => self.docs.push(YValue::BadValue(Loc::new(&mark))),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid, _) => {
                self.doc_stack
                    .push((YValue::Array(Vec::new(), Loc::new(&mark)), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node, mark)?;
            }
            Event::MappingStart(aid, _) => {
                self.doc_stack
                    .push((YValue::Hash(Hash::new(), Loc::new(&mark)), aid));
                self.key_stack.push(YValue::BadValue(Loc::new(&mark)));
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node, mark)?;
            }
            Event::Scalar(v, style, aid, tag) => {
                let node = if style != TScalarStyle::Plain {
                    YValue::String(v, Loc::new(&mark))
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
                                    Err(_) => YValue::BadValue(Loc::new(&mark)),
                                    Ok(v) => YValue::Boolean(v, Loc::new(&mark)),
                                }
                            }
                            "int" => match v.parse::<i64>() {
                                Err(_) => YValue::BadValue(Loc::new(&mark)),
                                Ok(v) => YValue::Integer(v, Loc::new(&mark)),
                            },
                            "float" => match parse_f64(&v) {
                                Some(_) => YValue::Real(v, Loc::new(&mark)),
                                None => YValue::BadValue(Loc::new(&mark)),
                            },
                            "null" => match v.as_ref() {
                                "~" | "null" => YValue::Null(Loc::new(&mark)),
                                _ => YValue::BadValue(Loc::new(&mark)),
                            },
                            _ => YValue::String(v, Loc::new(&mark)),
                        }
                    } else {
                        YValue::String(v, Loc::new(&mark))
                    }
                } else {
                    // Datatype is not specified, or unrecognized
                    YValue::from_str(&v, &mark)
                };

                self.insert_new_node((node, aid), mark)?;
            }
            _ => unreachable!(),
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
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (YValue::Array(ref mut v, _), _) => v.push(node.0),
                (YValue::Hash(ref mut h, _), _) => {
                    let cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = YValue::BadValue(Loc::new(&mark));
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

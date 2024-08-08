use std::{collections::BTreeMap, convert::TryFrom, mem, ops::Index, ops::IndexMut};

use yaml_rust2::parser::{Event, MarkedEventReceiver};
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
}

pub fn hello() {
    println!("Hello, world!");
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
}

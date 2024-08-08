use std::ops::{Deref, DerefMut};

use dashmap::DashMap;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionResponse, CompletionTriggerKind, Position,
};

#[derive(Debug, Clone, Default)]
pub struct AutoCompleteCache(DashMap<String, AutoCompleteTokenCache>);

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AutoCompleteTokenCache {
    value: String,
    position: Position,
    items: Vec<CompletionItem>,
}

impl Deref for AutoCompleteCache {
    type Target = DashMap<String, AutoCompleteTokenCache>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AutoCompleteCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AutoCompleteCache {
    pub fn add_entry(
        &self,
        uri_path: String,
        position: &Position,
        response: &CompletionResponse,
        value: &str,
    ) {
        if let CompletionResponse::Array(items) = response {
            self.0.insert(
                uri_path,
                AutoCompleteTokenCache {
                    value: value.to_string(),
                    position: *position,
                    items: items.clone(),
                },
            );
        }
    }

    pub fn get_cached_items(
        &self,
        uri_path: &str,
        position: &Position,
        context: &Option<CompletionContext>,
    ) -> Option<(CompletionResponse, String)> {
        let entry = self.0.get(uri_path)?;
        let trigger_char = get_trigger_char(context)?;

        if entry.value().position.line != position.line
            || entry.value().position.character + (trigger_char.len() as u32) != position.character
        {
            return None;
        }

        let mut value = entry.value().value.clone();
        value.push_str(&trigger_char);

        let items = entry
            .value()
            .items
            .clone()
            .into_iter()
            .filter(|item| item.label.starts_with(&value))
            .collect::<Vec<_>>();

        Some((CompletionResponse::Array(items), value))
    }
}

fn get_trigger_char(context: &Option<CompletionContext>) -> Option<String> {
    let context = context.as_ref()?;
    if context.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER {
        return context.trigger_character.clone();
    }
    None
}

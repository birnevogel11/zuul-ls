use tower_lsp::lsp_types::{Location, SymbolInformation, SymbolKind};

use super::symbols::ZuulSymbol;
use crate::parser::common::StringLoc;

pub fn new_symbol_information(
    name: String,
    location: Location,
    kind: SymbolKind,
) -> SymbolInformation {
    SymbolInformation {
        name,
        location,
        kind,
        container_name: None,
        tags: None,
        deprecated: None,
    }
}

pub fn query_workspace_symbols(symbols: &ZuulSymbol, query: &str) -> Vec<SymbolInformation> {
    let mut si = Vec::new();

    si.extend(symbols.jobs().iter().flat_map(|entry| {
        let name = entry.key();
        let locs = entry.value();

        locs.iter()
            .map(|loc| {
                new_symbol_information(name.to_string(), loc.clone().into(), SymbolKind::CLASS)
            })
            .collect::<Vec<_>>()
    }));

    si.extend(
        symbols
            .vars()
            .to_print_list()
            .into_iter()
            .map(|var_print_info| {
                new_symbol_information(
                    var_print_info.name.value.to_string(),
                    var_print_info.name.into(),
                    SymbolKind::VARIABLE,
                )
            }),
    );

    si.extend(symbols.role_dirs().iter().map(|entry| {
        let name = entry.key();
        let path = entry.value();

        new_symbol_information(
            name.to_string(),
            StringLoc::from_simple(name, path).into(),
            SymbolKind::FUNCTION,
        )
    }));

    si.extend(symbols.project_templates().iter().map(|entry| {
        new_symbol_information(
            entry.key().to_string(),
            entry.value().clone().into(),
            SymbolKind::MODULE,
        )
    }));

    if query.is_empty() {
        si
    } else {
        si.into_iter()
            .filter(|si| si.name.starts_with(query))
            .collect::<Vec<_>>()
    }
}

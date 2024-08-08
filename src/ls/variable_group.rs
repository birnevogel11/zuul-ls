use dashmap::mapref::entry::Entry;

use crate::parser::variable::VariableGroup;

/// Locate variable group by variable stack recursively.
pub fn process_var_group<T, U>(
    value: &str,
    var_stack: &[String],
    var_group: &VariableGroup,
    stack_idx: usize,
    process_func: T,
) -> Option<U>
where
    T: Fn(&str, &VariableGroup) -> Option<U>,
{
    if stack_idx != var_stack.len() {
        let var = var_stack[stack_idx].as_str();
        if var_group.contains_key(var) {
            if let Entry::Occupied(e) = var_group.entry(var.to_string()) {
                let m = &e.get().members;
                return process_var_group(value, var_stack, m, stack_idx + 1, process_func);
            }
        }
        None
    } else {
        process_func(value, var_group)
    }
}

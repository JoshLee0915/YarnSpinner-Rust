use crate::prelude::*;
use yarn_slinger_core::prelude::*;
use yarn_slinger_core::types::Type;

pub(crate) fn add_tracking_declarations(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    let tracking_declarations: Vec<_> = state
        .tracking_nodes
        .iter()
        .map(|node| {
            Declaration::default()
                .with_default_value(0.)
                .with_name(Library::generate_unique_visited_variable_for_node(node))
                .with_type(Type::Number)
                .with_description(format!(
                    "The generated variable for tracking visits of node {node}"
                ))
        })
        .collect();

    // adding the generated tracking variables into the declaration list
    // this way any future variable storage system will know about them
    // if we didn't do this later stages wouldn't be able to interface with them
    state
        .known_variable_declarations
        .extend(tracking_declarations.clone());
    state
        .derived_variable_declarations
        .extend(tracking_declarations);
    state
}
use crate::prelude::*;
use std::collections::HashSet;
use yarn_slinger_core::prelude::*;
use yarn_slinger_core::types::{Type, TypeFormat};

pub(crate) fn add_initial_value_registrations(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    // Last step: take every variable declaration we found in all
    // of the inputs, and create an initial value registration for
    // it.
    let Ok(compilation) = state.result.as_mut().unwrap().as_mut() else {
        return state;
    };

    let declarations = state
        .known_variable_declarations
        .iter()
        .filter(|decl| !matches!(decl.r#type, Some(Type::Function(_))))
        .filter(|decl| decl.r#type.is_some());

    for declaration in declarations {
        let Some(default_value) = declaration.default_value.clone() else {
             compilation.diagnostics.push(
                 Diagnostic::from_message(
                     format!("Variable declaration {} (type {}) has a null default value. This is not allowed.", declaration.name, declaration.r#type.format())));
             continue;
         };
        if let Some(ref mut program) = compilation.program {
            let value = match declaration.r#type.as_ref().unwrap() {
                Type::String => Operand::from(String::try_from(default_value).unwrap()),
                Type::Number => Operand::from(f32::try_from(default_value).unwrap()),
                Type::Boolean => Operand::from(bool::try_from(default_value).unwrap()),
                _ => panic!("Cannot create initial value registration for type {}. This is a bug. Please report it at https://github.com/yarn-slinger/yarn_slinger/issues/new", declaration.r#type.format()),
            };
            program
                .initial_values
                .insert(declaration.name.clone(), value);
        }
    }
    compilation.declarations = state.derived_variable_declarations.clone();
    let mut unique_diagnostics: HashSet<Diagnostic> =
        HashSet::from_iter(state.diagnostics.clone().into_iter());
    let mut ordered_unique_diagnostics = Vec::new();

    // preserve order
    for diagnostic in compilation.diagnostics.iter() {
        if unique_diagnostics.contains(diagnostic) {
            ordered_unique_diagnostics.push(diagnostic.clone());
            unique_diagnostics.remove(diagnostic);
        }
    }
    compilation.diagnostics = ordered_unique_diagnostics;
    state
}
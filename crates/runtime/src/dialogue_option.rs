//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/Dialogue.cs>, which we split off into multiple files

use crate::prelude::*;

/// An option to be presented to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueOption {
    /// The [`Line`] that should be presented to the user for this option.
    ///
    /// See the documentation for the [`Line`] struct for information on how to prepare a line before presenting it to the user.
    pub line: Line,

    /// The identifying number for this option.
    ///
    /// When the user selects this option, this value should be used as the parameter for [`Dialogue::set_selected_option`].
    pub id: OptionId,

    /// The name of the node that will be run if this option is selected.
    ///
    /// The value of this property not be valid if this is a shortcut option.
    pub destination_node: String,

    /// Gets a value indicating whether the player should be permitted to select this option.
    ///
    /// If this value is `false`, this option had a line condition on it that failed.
    /// The option will still be delivered to the game, but, depending on the needs of the game,
    /// the game may decide to not allow the player to select it, or not offer it to the player at all.
    ///
    /// This is intended for situations where games wish to show options that the player _could_ have taken,
    /// if some other condition had been met (e.g. having enough "charisma" points).
    pub is_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionId(pub(crate) usize);

impl OptionId {
    /// Constructs a new `OptionId` from the given value.
    /// A user is supposed to use the `OptionId`s constructed by the [`Dialogue`] and not create their own.
    ///
    /// So, only use this method for debugging purposes.
    pub fn construct_for_debugging(value: usize) -> Self {
        Self(value)
    }
}
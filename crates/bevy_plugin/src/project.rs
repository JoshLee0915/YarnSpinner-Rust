use crate::prelude::*;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
pub(crate) use compilation::{
    RecompileLoadedYarnFilesEvent, YarnFilesBeingLoaded, YarnProjectConfigToLoad,
};
use std::fmt::Debug;
use std::iter;

mod compilation;

pub(crate) fn project_plugin(app: &mut App) {
    app.fn_plugin(compilation::project_compilation_plugin)
        .add_event::<LoadYarnProjectEvent>();
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, SystemSet)]
pub(crate) struct CompilationSystemSet;

/// The compiled Yarn project built from the Yarn files passed to the [`YarnSlingerPlugin`], or, in the deferred loading case, the Yarn files passed to the [`LoadYarnProjectEvent`].
/// This [`Resource`](bevy::prelude::Resource) is inserted into the world automatically for you once all files have been loaded and compiled. You can react to this by configuring a system like this:
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_yarn_slinger::prelude::*;
/// # let mut app = App::new();
/// app.add_system(setup_dialogue_runners.run_if(resource_added::<YarnProject>()));
///
/// fn setup_dialogue_runners(mut commands: Commands, project: Res<YarnProject>) {
///    commands.spawn(project.create_dialogue_runner());
/// }
/// ```
#[derive(Resource)]
pub struct YarnProject {
    pub(crate) yarn_files: HashSet<Handle<YarnFile>>,
    pub(crate) compilation: Compilation,
    pub(crate) localizations: Option<Localizations>,
    pub(crate) asset_server: AssetServer,
    pub(crate) metadata: HashMap<LineId, Vec<String>>,
    pub(crate) watching_for_changes: bool,
    pub(crate) file_generation_mode: FileGenerationMode,
}

impl Debug for YarnProject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YarnProject")
            .field("yarn_files", &self.yarn_files)
            .field("compilation", &self.compilation)
            .field("localizations", &self.localizations)
            .field("asset_server", &())
            .field("metadata", &self.metadata)
            .field("watching_for_changes", &self.watching_for_changes)
            .finish()
    }
}

impl YarnProject {
    /// Iterates over the [`YarnFile`]s that were used to compile this project. These will be the files passed to the [`YarnSlingerPlugin`] or the [`LoadYarnProjectEvent`].
    pub fn yarn_files(&self) -> impl Iterator<Item = &Handle<YarnFile>> {
        self.yarn_files.iter()
    }

    /// Returns the underlying [`Compilation`] of this project. This is advanced functionality.
    pub fn compilation(&self) -> &Compilation {
        &self.compilation
    }

    /// Returns the [`Localizations`] of this project, if any. These come from [`YarnSlingerPlugin::with_localizations`] or [`LoadYarnProjectEvent::with_localizations`].
    pub fn localizations(&self) -> Option<&Localizations> {
        self.localizations.as_ref()
    }

    /// Constructs a [`DialogueRunner`] from this project using all defaults of [`DialogueRunnerBuilder`] .
    /// This is a convenience method for calling [`DialogueRunnerBuilder::build`] on an unconfigured builder returned by [`YarnProject::build_dialogue_runner`].
    pub fn create_dialogue_runner(&self) -> DialogueRunner {
        self.build_dialogue_runner().build()
    }

    /// Constructs a [`DialogueRunnerBuilder`] that can be used to customize the construction of a [`DialogueRunner`] from this project.
    pub fn build_dialogue_runner(&self) -> DialogueRunnerBuilder {
        DialogueRunnerBuilder::from_yarn_project(self)
    }

    /// Returns the metadata associated with the given [`LineId`], if any. This can also be accessed on a given [`LocalizedLine`] via its `metadata` field.
    pub fn line_metadata(&self, line_id: &LineId) -> Option<&[String]> {
        self.metadata.get(line_id).map(|v| v.as_slice())
    }

    /// Returns the headers associated with the given node, if it exists.
    pub fn headers_for_node(&self, node_name: &str) -> Option<HashMap<&str, Vec<&str>>> {
        self.compilation
            .program
            .as_ref()
            .unwrap()
            .nodes
            .get(node_name)?
            .headers
            .iter()
            .fold(HashMap::new(), |mut map, header| {
                map.entry(header.key.as_str())
                    .or_insert_with(Vec::new)
                    .push(header.value.as_str());
                map
            })
            .into()
    }
}

/// Used to late initialize a [`YarnProject`] with a set of Yarn files when using [`YarnSlingerPlugin::deferred`].
/// If you know the yarn files at the start of the game, you should use [`YarnSlingerPlugin::with_yarn_sources`] instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadYarnProjectEvent {
    pub(crate) localizations: Option<Localizations>,
    pub(crate) yarn_files: HashSet<YarnFileSource>,
    pub(crate) file_generation_mode: FileGenerationMode,
}

impl Default for LoadYarnProjectEvent {
    fn default() -> Self {
        Self {
            localizations: None,
            yarn_files: HashSet::from([YarnFileSource::Folder(DEFAULT_ASSET_DIR.into())]),
            file_generation_mode: default(),
        }
    }
}

impl LoadYarnProjectEvent {
    /// See [`YarnSlingerPlugin::new`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// See [`YarnSlingerPlugin::with_yarn_sources`].
    #[must_use]
    pub fn with_yarn_sources<T, U>(yarn_files: T) -> Self
    where
        T: IntoIterator<Item = U>,
        U: Into<YarnFileSource>,
    {
        let yarn_files = yarn_files
            .into_iter()
            .map(|yarn_file| yarn_file.into())
            .collect();
        Self {
            localizations: None,
            yarn_files,
            file_generation_mode: default(),
        }
    }

    #[must_use]
    pub fn with_yarn_source(yarn_file_source: impl Into<YarnFileSource>) -> Self {
        Self::with_yarn_sources(iter::once(yarn_file_source))
    }

    /// See [`YarnSlingerPlugin::add_yarn_source`].
    #[must_use]
    pub fn add_yarn_source(mut self, yarn_file: impl Into<YarnFileSource>) -> Self {
        self.yarn_files.insert(yarn_file.into());
        self
    }

    /// See [`YarnSlingerPlugin::add_yarn_sources`].
    #[must_use]
    pub fn add_yarn_sources(
        mut self,
        yarn_files: impl IntoIterator<Item = impl Into<YarnFileSource>>,
    ) -> Self {
        self.yarn_files
            .extend(yarn_files.into_iter().map(|yarn_file| yarn_file.into()));
        self
    }

    /// See [`YarnSlingerPlugin::with_localizations`].
    #[must_use]
    pub fn with_localizations(mut self, localizations: impl Into<Option<Localizations>>) -> Self {
        let localizations = localizations.into();
        self.localizations = localizations;
        self
    }

    #[must_use]
    pub fn with_file_generation_mode(mut self, file_generation_mode: FileGenerationMode) -> Self {
        self.file_generation_mode = file_generation_mode;
        if cfg!(any(target_arch = "wasm32", target_os = "android")) {
            assert_ne!(self.file_generation_mode, FileGenerationMode::Development,
               "Failed to build Yarn Slinger plugin: File generation mode \"Development\" is not supported on this target.");
        }
        self
    }
}

impl<T, U> From<T> for LoadYarnProjectEvent
where
    T: IntoIterator<Item = U>,
    U: Into<YarnFileSource>,
{
    fn from(yarn_files: T) -> Self {
        Self::with_yarn_sources(yarn_files)
    }
}

#[derive(Debug, Clone, Resource, Default)]
pub(crate) struct WatchingForChanges(pub(crate) bool);

pub(crate) const DEFAULT_ASSET_DIR: &str = "dialogue";

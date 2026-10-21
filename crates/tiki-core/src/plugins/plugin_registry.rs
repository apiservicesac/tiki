use super::{ConnectionKind, Plugin, PluginManifest};

/// The installed plugins.
#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a plugin; a later plugin with the same id replaces the earlier one.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let plugin_id = plugin.manifest().id.clone();
        self.plugins
            .retain(|existing| existing.manifest().id != plugin_id);
        self.plugins.push(plugin);
    }

    pub fn find(&self, plugin_id: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|plugin| plugin.manifest().id == plugin_id)
            .map(AsRef::as_ref)
    }

    pub fn manifests(&self) -> impl Iterator<Item = &PluginManifest> {
        self.plugins.iter().map(|plugin| plugin.manifest())
    }

    /// Plugins that can open the given kind of connection.
    pub fn offering(&self, kind: ConnectionKind) -> impl Iterator<Item = &PluginManifest> {
        self.manifests()
            .filter(move |manifest| manifest.offers(kind))
    }
}

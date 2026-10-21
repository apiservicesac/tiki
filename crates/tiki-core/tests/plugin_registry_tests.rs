use std::collections::BTreeMap;

use tiki_core::plugins::{
    ConnectionConfig, ConnectionKind, Plugin, PluginManifest, PluginRegistry,
};

/// A plugin that only offers ticket sources, like a future Slack plugin.
struct TicketOnlyPlugin {
    manifest: PluginManifest,
}

impl TicketOnlyPlugin {
    fn boxed(id: &str) -> Box<dyn Plugin> {
        Box::new(Self {
            manifest: PluginManifest {
                id: id.into(),
                display_name: id.to_uppercase(),
                connection_kinds: vec![ConnectionKind::TicketSource],
            },
        })
    }
}

impl Plugin for TicketOnlyPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

fn sample_config() -> ConnectionConfig {
    ConnectionConfig {
        connection_id: "support-channel".into(),
        display_name: "Support".into(),
        base_url: "https://example.slack.com".into(),
        username: "tiki".into(),
        credential_key: "tiki/slack/support-channel".into(),
        options: BTreeMap::default(),
    }
}

#[test]
fn the_registry_lists_plugins_by_connection_kind() {
    let mut registry = PluginRegistry::new();
    registry.register(TicketOnlyPlugin::boxed("slack"));

    assert_eq!(registry.offering(ConnectionKind::TicketSource).count(), 1);
    assert_eq!(registry.offering(ConnectionKind::ClientInstance).count(), 0);
}

#[test]
fn registering_the_same_id_replaces_the_plugin() {
    let mut registry = PluginRegistry::new();
    registry.register(TicketOnlyPlugin::boxed("slack"));
    registry.register(TicketOnlyPlugin::boxed("slack"));

    assert_eq!(registry.manifests().count(), 1);
}

#[test]
fn a_plugin_refuses_connection_kinds_it_does_not_offer() {
    let mut registry = PluginRegistry::new();
    registry.register(TicketOnlyPlugin::boxed("slack"));

    let plugin = registry.find("slack").expect("registered plugin");
    assert!(plugin.open_client_instance(&sample_config()).is_err());
}

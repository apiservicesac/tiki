use tauri::menu::{Menu, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle};
use tiki_core::i18n::Translator;

use crate::desktop_config::DesktopConfig;
use crate::island_window::IslandWindow;
use crate::shell_events::ShellEvent;

/// Menu entries of the tray icon.
#[derive(Debug, Clone, Copy)]
enum TrayAction {
    Open,
    Capture,
    Settings,
    Quit,
}

impl TrayAction {
    const ALL: [TrayAction; 4] = [
        TrayAction::Open,
        TrayAction::Capture,
        TrayAction::Settings,
        TrayAction::Quit,
    ];

    fn id(self) -> &'static str {
        match self {
            TrayAction::Open => "open",
            TrayAction::Capture => "capture",
            TrayAction::Settings => "settings",
            TrayAction::Quit => "quit",
        }
    }

    fn label_key(self) -> &'static str {
        match self {
            TrayAction::Open => "tray.open",
            TrayAction::Capture => "tray.capture",
            TrayAction::Settings => "tray.settings",
            TrayAction::Quit => "tray.quit",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}

/// The icon next to the Windows clock, with its menu.
pub struct IslandTray;

impl IslandTray {
    pub fn install(app: &App, translator: &Translator) -> tauri::Result<()> {
        let items = TrayAction::ALL
            .into_iter()
            .map(|action| {
                MenuItem::with_id(
                    app,
                    action.id(),
                    translator.text(action.label_key()),
                    true,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<Vec<_>>>()?;
        let item_references: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = items
            .iter()
            .map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
            .collect();
        let menu = Menu::with_items(app, &item_references)?;

        let mut builder = TrayIconBuilder::with_id(DesktopConfig::TRAY_ID)
            .tooltip(translator.text("tray.tooltip"))
            .menu(&menu)
            .on_menu_event(Self::handle_menu_event);
        if let Some(icon) = app.default_window_icon() {
            builder = builder.icon(icon.clone());
        }
        builder.build(app)?;
        Ok(())
    }

    // The signature is fixed by `TrayIconBuilder::on_menu_event`.
    #[allow(clippy::needless_pass_by_value)]
    fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
        let shell_event = match TrayAction::from_id(event.id().as_ref()) {
            Some(TrayAction::Open) => ShellEvent::ToggleIsland,
            Some(TrayAction::Capture) => ShellEvent::OpenCapture,
            Some(TrayAction::Settings) => ShellEvent::OpenSettings,
            Some(TrayAction::Quit) => {
                app.exit(0);
                return;
            }
            None => return,
        };
        if let Some(island) = IslandWindow::find(app) {
            island.bring_forward();
        }
        shell_event.emit(app);
    }
}

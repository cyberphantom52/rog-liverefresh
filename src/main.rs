mod display_config;

use crate::display_config::ApplyConfig;
use display_config::display_config::DisplayConfigProxy;
use zbus::export::futures_util::{future::try_join, StreamExt};
use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy(assume_defaults = true)]
trait UPower {
    #[dbus_proxy(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}

#[dbus_proxy(
    interface = "org.gnome.ScreenSaver",
    default_service = "org.gnome.ScreenSaver",
    default_path = "/org/gnome/ScreenSaver"
)]
trait ScreenSaver {
    /// WakeUpScreen signal
    #[dbus_proxy(signal)]
    fn wake_up_screen(&self) -> zbus::Result<()>;
}

async fn update_display_config(proxy: &DisplayConfigProxy<'_>, on_battery: bool) -> Result<()> {
    let state = proxy.get_current_state().await?;
    let builtin_pm = state.get_builtin_physical_monitor().unwrap();
    let current_mode = builtin_pm.get_current_mode().id.clone();
    let new_mode = builtin_pm.get_alternate_mode(on_battery).id.clone();

    if current_mode == new_mode {
        return Ok(());
    };

    let config = ApplyConfig::from(state, new_mode);
    proxy
        .apply_monitors_config(
            config.serial,
            config.method,
            config.logical_monitors,
            config.properties,
        )
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let system_connection = Connection::system().await?;
    let session_connection = Connection::session().await?;

    let display_config_proxy = DisplayConfigProxy::new(&session_connection).await?;

    let upower_proxy = UPowerProxy::new(&system_connection).await?;
    let mut battery_stream = upower_proxy.receive_on_battery_changed().await;
    
    let screen_saver_proxy = ScreenSaverProxy::new(&session_connection).await?;
    let mut screen_wakeup_stream = screen_saver_proxy.receive_wake_up_screen().await?;

    try_join(
        async {
            while let Some(on_battery) = battery_stream.next().await {
                let value = on_battery.get().await?;
                update_display_config(&display_config_proxy, value).await?;
            }
            Ok::<(), zbus::Error>(())
        },
        async {
            while let Some(_) = screen_wakeup_stream.next().await {
                let on_battery = upower_proxy.on_battery().await?;
                update_display_config(&display_config_proxy, on_battery).await?;
            }
            Ok(())
        },
    )
    .await?;
    // println!("{:#?}", state);
    Ok(())
}

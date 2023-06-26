mod display_config;

use crate::display_config::ApplyConfig;
use display_config::display_config::DisplayConfigProxy;
use tokio::time::{sleep, Duration};
use zbus::export::futures_util::{future::try_join, StreamExt};
use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy(assume_defaults = true)]
trait UPower {
    #[dbus_proxy(property)]
    fn on_battery(&self) -> zbus::Result<bool>;

    #[dbus_proxy(property)]
    fn lid_is_closed(&self) -> zbus::Result<bool>;
}

async fn update_display_config(proxy: &DisplayConfigProxy<'_>) -> Result<()> {
    let state = proxy.get_current_state().await?;
    let builtin_pm = state.get_builtin_physical_monitor();
    let current_mode = builtin_pm.get_current_mode().await.id.clone();
    let new_mode = builtin_pm.get_alternate_mode().await.id.clone();

    if current_mode == new_mode {
        return Ok(());
    };

    let config = ApplyConfig::from(state).await;
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
    let conn_sess = Connection::session().await?;
    let display_proxy = DisplayConfigProxy::new(&conn_sess).await?;

    let conn_sys = Connection::system().await?;
    let upower_proxy = UPowerProxy::new(&conn_sys).await?;

    let mut lid_stream = upower_proxy.receive_lid_is_closed_changed().await;
    let mut battery_stream = upower_proxy.receive_on_battery_changed().await;

    try_join(
        async {
            while let Some(_) = battery_stream.next().await {
                update_display_config(&display_proxy).await?;
            }
            Ok::<(), zbus::Error>(())
        },
        async {
            while let Some(lid_state) = lid_stream.next().await {
                let value = lid_state.get().await?;
                if !value {
                    // Delay the update to give the display time to turn on
                    sleep(Duration::from_secs(1)).await;
                    update_display_config(&display_proxy).await?;
                }
            }
            Ok(())
        },
    )
    .await?;
    // println!("{:#?}", state);
    Ok(())
}

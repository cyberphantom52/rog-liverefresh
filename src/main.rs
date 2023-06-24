mod display_config;

use crate::display_config::ApplyConfig;
use display_config::display_config::DisplayConfigProxy;
use zbus::export::futures_util::StreamExt;
use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy]
trait UPower {
    #[dbus_proxy(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let conn_sess = Connection::session().await?;
    let display_proxy = DisplayConfigProxy::new(&conn_sess).await?;

    let conn_sys = Connection::system().await?;
    let upower_proxy = UPowerProxy::new(&conn_sys).await?;

    let mut stream = upower_proxy.receive_on_battery_changed().await;
    while let Some(_) = stream.next().await {
        let state = display_proxy.get_current_state().await?;
        
        let builtin_pm = state.get_builtin_physical_monitor();
        let current_mode = builtin_pm.get_current_mode()
            .await
            .id
            .clone();
        let new_mode = builtin_pm.get_alternate_mode()
            .await
            .id
            .clone();
        
        if current_mode == new_mode {continue};
        
        let config = ApplyConfig::from(state).await;
        display_proxy
            .apply_monitors_config(
                config.serial,
                config.method,
                config.logical_monitors,
                config.properties,
            )
            .await?;
    }
    // println!("{:#?}", state);
    Ok(())
}

mod display_config;

use zbus::{Connection, Result, dbus_proxy};
use zbus::export::futures_util::StreamExt;
use display_config::display_config::DisplayConfigProxy;
use crate::display_config::ApplyConfig;

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
        let config = ApplyConfig::from(state).await;
        display_proxy.apply_monitors_config(config.serial, config.method, config.logical_monitors, config.properties).await?;
    }
    // println!("{:#?}", state);
    Ok(())
}

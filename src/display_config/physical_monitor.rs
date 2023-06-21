use serde::{Serialize, Deserialize};
use zbus::{zvariant::{Type, OwnedValue}, Connection};
use std::collections::HashMap;

use crate::UPowerProxy;

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct Monitor {
    pub connector: String,
    
    pub vendor: String,
    
    pub product: String,
    
    pub serial: String,
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct Mode {
    pub id: String,
    
    pub width: i32,
    
    pub height: i32,
    
    pub refresh_rate: f64,
    
    pub preferred_scale: f64,
    
    pub supported_scales: Vec<f64>,
    
    /*
        optional properties, including:
            - "is-current" (b): the mode is currently active mode
            - "is-preferred" (b): the mode is the preferred mode
            - "is-interlaced" (b): the mode is an interlaced mode
    */
    pub properties: HashMap<String, OwnedValue>,
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct PhysicalMonitor {
    pub monitor: Monitor,
    
    pub modes: Vec<Mode>,
	
    /*  
        optional properties, including:
	        - "width-mm" (i): physical width of monitor in millimeters
	        - "height-mm" (i): physical height of monitor in millimeters
	        - "is-underscanning" (b): whether underscanning is enabled
		    		      (absence of this means underscanning
		    		      not being supported)
	        - "max-screen-size" (ii): the maximum size a screen may have
		    		      (absence of this means unlimited screen
		    		      size)
	        - "is-builtin" (b): whether the monitor is built in, e.g. a
		    		laptop panel (absence of this means it is
		    		not built in)
	        - "display-name" (s): a human readable display name of the monitor
	        - "privacy-screen-state" (bb): the state of the privacy screen
		    		     (absence of this means it is not being
		    		     supported) first value indicates whether
		    		     it's enabled and second value whether it's
		    		     hardware locked (and so can't be changed
		    		     via gsettings)

            Possible mode flags:
                1 : preferred mode
                2 : current mode
      */
    pub properties: HashMap<String, OwnedValue>,
}

impl PhysicalMonitor {
    pub fn is_builtin(&self) -> bool  {
        self.properties
            .get("is-builtin")
            .unwrap_or(&OwnedValue::from(false))
            .try_into()
            .unwrap()
    }

    pub fn get_current_mode(&self) -> &Mode {
        self.modes
            .iter()
            .find(|mode| mode.properties
                .get("is-current")
                .unwrap_or(&OwnedValue::from(false))
                .try_into()
                .unwrap()
            ).unwrap()
    }

    // Find the lowest refresh rate mode if highest refresh rate mode is set
    // else find the highest refresh rate mode
    pub async fn get_alternate_mode(&self) -> &Mode {
        let connection = Connection::system().await.unwrap();
        let proxy = UPowerProxy::new(&connection).await.unwrap();
        let on_battery = proxy.on_battery().await.unwrap();

        let curr_mode = self.get_current_mode();
        let modes = self.modes
            .iter()
            .filter(|mode| mode.width == curr_mode.width && mode.height == curr_mode.height)
            .collect::<Vec<&Mode>>();
        
        println!("modes: {:#?}", modes);
        if !on_battery {
            modes.first().unwrap()
        } else {
            modes.last().unwrap()
        }
    }

    pub fn get_connector(&self) -> String {
        self.monitor.connector.clone()
    }    
}
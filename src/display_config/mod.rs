pub mod display_config;
pub mod logical_monitor;
pub mod physical_monitor;

use self::logical_monitor::{LogicalMonitor, Transform};
use self::physical_monitor::PhysicalMonitor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zbus::zvariant::{OwnedValue, Type};

#[derive(Debug, Type, Serialize, Deserialize)]
pub enum Method {
    VERIFY,
    TEMPORARY,
    PERSISTENT,
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct ApplyMonitor {
    pub connector: String,
    pub mode_id: String,

    /*  monitor properties, including:
            - "enable_underscanning" (b): enable monitor underscanning;
    */
    pub properties: HashMap<String, OwnedValue>,
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct ApplyLogicalMonitor {
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub transform: Transform,
    pub primary: bool,
    pub monitors: Vec<ApplyMonitor>,
}

impl ApplyLogicalMonitor {
    pub fn from(logical_monitor: &LogicalMonitor, connector: String, new_mode_id: String) -> Self {
        Self {
            x: logical_monitor.x,
            y: logical_monitor.y,
            scale: logical_monitor.scale,
            transform: logical_monitor.transform,
            primary: logical_monitor.primary,
            monitors: vec![ApplyMonitor {
                connector: connector,
                mode_id: new_mode_id,
                properties: HashMap::new(),
            }],
        }
    }
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct State {
    pub serial: u32,
    pub physical_monitors: Vec<PhysicalMonitor>,
    pub logical_monitors: Vec<LogicalMonitor>,

    /* Possible @properties are:
         "layout-mode" (u): Represents in what way logical monitors are laid
                             out on the screen. The layout mode can be either
                             of the ones listed below. Absence of this property
                             means the layout mode cannot be changed, and that
                             "logical" mode is assumed to be used.
                * 1 : logical  - the dimension of a logical monitor is derived from
                         the monitor modes associated with it, then scaled
                         using the logical monitor scale.
                * 2 : physical - the dimension of a logical monitor is derived from
                         the monitor modes associated with it.
         "supports-changing-layout-mode" (b): True if the layout mode can be
                           changed. Absence of this means the
                           layout mode cannot be changed.
         "global-scale-required" (b): True if all the logical monitors must
                           always use the same scale. Absence of
                           this means logical monitor scales can
                           differ.
         "legacy-ui-scaling-factor" (i): The legacy scaling factor traditionally
                          used to scale X11 clients (commonly
                          communicated via the
                          Gdk/WindowScalingFactor XSetting entry).
    */
    pub properties: HashMap<String, OwnedValue>,
}

impl State {
    pub fn get_builtin_physical_monitor(&self) -> Option<&PhysicalMonitor> {
        self.physical_monitors.iter().find(|pm| pm.is_builtin())
    }

    pub fn get_logical_monitor(&self, connector: &str) -> Option<&LogicalMonitor> {
        self.logical_monitors
            .iter()
            .find(|lm| {
                lm.monitors
                    .iter()
                    .find(|monitor| monitor.connector == connector)
                    .is_some()
            })
            .map(|lm| lm)
    }
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct ApplyConfig {
    pub serial: u32,
    pub method: Method,
    pub logical_monitors: Vec<ApplyLogicalMonitor>,

    /* Possible properties are:
        * "layout-mode" (u): layout mode the passed configuration is in; may
                 only be set when changing the layout mode is
                 supported (see GetCurrentState).
    */
    pub properties: HashMap<String, OwnedValue>,
}

impl ApplyConfig {
    pub fn from(state: State, new_mode_id: String) -> Self {
        let connector = state
            .get_builtin_physical_monitor()
            .unwrap()
            .get_connector();

        let logical_monitor = state.get_logical_monitor(&connector).unwrap();

        Self {
            serial: state.serial,
            method: Method::TEMPORARY,
            logical_monitors: vec![ApplyLogicalMonitor::from(
                logical_monitor,
                connector,
                new_mode_id,
            )],
            properties: HashMap::new(),
        }
    }
}

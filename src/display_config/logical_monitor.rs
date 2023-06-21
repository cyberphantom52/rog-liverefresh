use serde::{Serialize, Deserialize};
use zbus::zvariant::{Type, OwnedValue};
use std::collections::HashMap;
use super::physical_monitor::Monitor;

#[derive(Clone, Copy, Debug, Type, Serialize, Deserialize)]
pub enum Transform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

#[derive(Debug, Type, Serialize, Deserialize)]
pub struct LogicalMonitor {
    pub x: i32,
    
    pub y: i32,
    
    pub scale: f64,

    pub transform: Transform,

    pub primary: bool,

    // physical monitors displaying this logical monitor
    pub monitors: Vec<Monitor>,

    // possibly other properties
    pub properties: HashMap<String, OwnedValue>,
}

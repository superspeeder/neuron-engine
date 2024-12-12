use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum QueueLabel {
    Graphics,
    Compute,
    Transfer,
    Presentation,
    VideoDecode,
    VideoEncode,
    Custom(&'static str),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct QueueRef {
    pub family: u32,
    pub index: u32,
}

pub type QueueLabels = HashMap<QueueLabel, Vec<QueueRef>>;
pub type UnlabeledQueues = HashMap<u32, HashSet<u32>>;


use crate::events::{HoverTarget, DragMode};

pub struct State {
    pub args: engyls::config::DisplayArgs,
    pub hover: HoverTarget,
    pub drag_mode: DragMode,
    pub drag_start_val_x: i32,
    pub drag_start_val_y: i32,
    pub drag_start_width: i32,
    pub drag_start_height: i32,
    pub capacity_chars: usize,
    pub line_height: i32,
    pub screen_width: f64,
    pub screen_height: f64,
}

pub const PREVIEW_QUOTE: &str = "The history of all hitherto existing society is the history of class \
struggles. Freeman and slave, patrician and plebeian, lord and serf, guild-master and journeyman, \
in a word, oppressor and oppressed, stood in constant opposition to one another, carried on an \
uninterrupted, now hidden, now open fight, a fight that each time ended, either in a revolutionary \
reconstitution of society at large, or in the common ruin of the contending classes. In the earlier \
epochs of history, we find almost everywhere a complicated arrangement of society into various \
orders, a manifold gradation of social rank. In ancient Rome we have patricians, knights, plebeians, \
slaves; in the Middle Ages, feudal lords, vassals, guild-masters, journeymen, apprentices, serfs; \
in almost all of these classes, again, subordinate gradations. The modern bourgeois society that has \
sprouted from the ruins of feudal society has not done away with class antagonisms.";

pub const PREVIEW_AUTHOR: &str = "— Karl Marx";

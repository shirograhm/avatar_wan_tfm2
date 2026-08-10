//! Tunables that belong to the *effect logic*.
//!
//! The champion's shell — stats, growth, action durations, cooldowns, ranges,
//! targeting and descriptions — lives in `champion/avatar_wan.data_champion`,
//! not here. What stays in Rust is only what the native effects compute with.
//!
//! Distances are world units (the map is 960,000 × 960,000 and a champion
//! radius is ~10,000); durations are ticks (60 ticks = 1 second).

use crate::element::Element;

/// Simulation tick rate.
pub const TICKS_PER_SECOND: f64 = 60.0;

/// Must match the mod folder name — the loader rejects a mismatch.
pub const MOD_ID: &str = "avatar_wan_tfm2";

// -- basic attack: Soul of Raava --
/// Percent of Wan's attack damage converted to magic damage.
/// His attack damage is dealt in full, but split down the middle: half lands
/// as magic and half as physical, so each half is mitigated by a different
/// resistance and neither armour nor magic resist alone answers him.
pub const ATTACK_MAGIC_SHARE: usize = 50;
pub const ATTACK_PHYSICAL_SHARE: usize = 50;

// -- elements --
/// Wan starts attuned to fire, the element he learned first, so his first
/// Avatar Cycle moves him to the next entry in `element::CYCLE`.
pub const STARTING_ELEMENT: Element = Element::Fire;

// Air: stacking attack speed on hit.
pub const AIR_STACK_BUFF: &str = "wan_air_stack";
pub const AIR_ATTACK_SPEED_PERCENT: i32 = 4;
pub const AIR_DURATION: usize = 2 * 60;
pub const AIR_MAX_STACKS: usize = 4;

// Water: heal on hit.
pub const WATER_HEAL: usize = 5;
pub const WATER_AP_RATIO: usize = 5;

// Earth: bonus physical damage, splashed around the target.
pub const EARTH_DAMAGE: usize = 10;
pub const EARTH_AP_RATIO: usize = 10;
/// Splash radius around the struck target. World units are 1000× the display
/// scale, so "40 range" in the tooltip is 40,000 here.
pub const EARTH_RADIUS: u64 = 40_000;

// Fire: burn over time on hit.
pub const BURN_DAMAGE: usize = 10;
pub const BURN_AD_RATIO: usize = 10;
pub const BURN_AP_RATIO: usize = 25;
/// Six ticks, one every half second, for three seconds total.
pub const BURN_TICKS: usize = 6;
pub const BURN_TICK_INTERVAL: usize = 30;

// -- skill2: Spirit Step --
/// The window itself: carries the movement speed and marks how long damage is
/// still being stored.
pub const STEP_BUFF: &str = "wan_spirit_step";
pub const STEP_MOVE_SPEED_PERCENT: i32 = 20;
pub const STEP_BUFF_DURATION: usize = 4 * 60;
/// Share of incoming damage banked during the window.
pub const STEP_STORE_PERCENT: usize = 80;
/// Share of the bank paid back as healing when the window ends.
pub const STEP_HEAL_PERCENT: usize = 100;
/// Prefix of the bookkeeping buff. Its name carries the running total and the
/// last HP reading — see `match_hook`, which is what reads and rewrites it.
pub const STEP_STORE_PREFIX: &str = "wan_step_store";

// -- ult: Harmonic Convergence --
pub const CONVERGENCE_BUFF: &str = "wan_harmonic_convergence";
pub const CONVERGENCE_DURATION: usize = 8 * 60;
pub const CONVERGENCE_SHIELD: usize = 250;
pub const CONVERGENCE_SHIELD_AP_RATIO: usize = 80;
pub const CONVERGENCE_SHIELD_HP_RATIO: usize = 8;
/// Full strength — what a proc is worth outside the ult, and what the element
/// Wan is actually channeling keeps during it.
pub const CONVERGENCE_BASE_SCALE: usize = 100;
/// During the ult his other three elements proc too, at reduced strength.
pub const CONVERGENCE_OFF_ELEMENT_SCALE: usize = 40;

/// Seconds to ticks.
#[allow(dead_code)]
pub fn ticks(seconds: f64) -> usize {
    (seconds * TICKS_PER_SECOND).round() as usize
}

/// `value * percent / 100`, rounded.
pub fn percent_of(value: usize, percent: usize) -> usize {
    (value * percent + 50) / 100
}

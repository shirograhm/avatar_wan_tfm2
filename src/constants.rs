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

// -- how the AI plays him --
/// Wan's champion id, as the AI boundary reports it — what `player_ai::matches`
/// compares against so the override touches nobody else's champion.
pub const CHAMPION_ID: &str = "avatar_wan";
/// Must match `attack.range` in the .data_champion. Duplicated because the AI
/// decides at a distance and the data file is not readable from here.
pub const ATTACK_RANGE: u64 = 60_000;
/// How far past his own range he will walk to start a fight. A quarter of a
/// range further than he can already shoot is a step forward, not a dive.
pub const AGGRO_ENGAGE_RANGE: u64 = 71_000;
/// The one thing that still pulls him out of Harmonic Convergence. The ult
/// otherwise suspends his health floor entirely, which is right while the
/// shield is doing its job and wrong once it is gone — below this he is dying
/// with the window still running, and a corpse lands no attacks at all.
pub const AGGRO_ULT_HP_FLOOR: usize = 15;
/// How far he will walk to find someone to spend Harmonic Convergence on.
/// Further than [`AGGRO_ENGAGE_RANGE`]: the ult is eight seconds of empowered
/// attacks on a long cooldown, and a second spent walking is a second of it
/// thrown away, so it is worth more of a step than a normal engage is.
pub const AGGRO_ULT_ENGAGE_RANGE: u64 = 90_000;
/// An enemy champion this close means he is in a fight. Creeps and towers do
/// not count: Spirit Step banks damage, and only champions threaten enough of
/// it to be worth the cooldown.
pub const AGGRO_COMBAT_RANGE: u64 = 90_000;
/// Below this, out of combat, he stops casting Spirit Step for the heal — the
/// bank cannot fill with nobody hitting him, so the cast is pure waste.
pub const AGGRO_LOW_HP: usize = 50;
/// Health percent he will keep trading at normally…
pub const AGGRO_HP_FLOOR: usize = 55;
/// …and while Spirit Step or Harmonic Convergence is running, when the damage
/// he takes is being paid back or absorbed and backing off wastes the window.
/// Still well above the default AI's threshold, but no longer a floor he only
/// reaches by having already lost the fight.
pub const AGGRO_COMMITTED_HP_FLOOR: usize = 30;

// -- elements --
/// Wan starts attuned to fire, the element he learned first, so his first
/// Avatar Cycle moves him to the next entry in `element::CYCLE`.
pub const STARTING_ELEMENT: Element = Element::Fire;

// Air: stacking attack speed on hit.
pub const AIR_STACK_BUFF: &str = "wan_air_stack";
pub const AIR_ATTACK_SPEED_PERCENT: i32 = 5;
pub const AIR_DURATION: usize = 2 * 60;
pub const AIR_MAX_STACKS: usize = 4;

// Water: heal on hit, as a share of Wan's *missing* health — so it scales
// with how much trouble he is in rather than with AP.
pub const WATER_MISSING_HP_PERCENT: usize = 5;
/// Statless marker that makes the `view_buffs` entry of the same name draw the
/// droplet burst where the attack landed — the same channel Earth's splash and
/// Fire's flame use, since a native effect cannot trigger a view effect
/// directly.
pub const WATER_SPLASH_VFX_BUFF: &str = "wan_water_splash";
/// Must match the run time of the `splash` tag in
/// `effects/wan_water_splash#anim.fanim` — six frames at 0.05s. Deliberately
/// shorter than Earth's: this one marks a single hit, not a shockwave.
pub const WATER_SPLASH_VFX_TICKS: usize = 18;

// Earth: a share of the basic attack itself, splashed around the target.
/// Percent of Wan's attack damage each splashed enemy takes. Dealt with the
/// same physical/magic split as the attack it copies, so Earth reads as the
/// attack spreading outward rather than as a separate source of damage.
pub const EARTH_ATTACK_SHARE: usize = 50;
/// Splash radius around the struck target. World units are 1000× the display
/// scale, so "40 range" in the tooltip is 40,000 here — two thirds of his
/// attack range.
pub const EARTH_RADIUS: u64 = 40_000;
/// Statless marker applied to each splashed enemy purely so the `view_buffs`
/// entry of the same name draws the splash on them. A native effect cannot
/// trigger a view effect directly; a buff is the channel that exists.
pub const EARTH_SPLASH_VFX_BUFF: &str = "wan_earth_splash";
/// Must match the run time of the `splash` tag in
/// `effects/wan_earth_splash#anim.fanim` — six frames at 0.1s. Too short and
/// the animation is cut off; too long and it holds on its last frame.
pub const EARTH_SPLASH_VFX_TICKS: usize = 36;

// Fire: burn over time on hit.
pub const BURN_DAMAGE: usize = 10;
pub const BURN_AD_RATIO: usize = 15;
pub const BURN_AP_RATIO: usize = 35;
/// Six ticks, one every half second, for three seconds total.
pub const BURN_TICKS: usize = 6;
pub const BURN_TICK_INTERVAL: usize = 30;
/// Statless marker that makes the `view_buffs` entry of the same name loop the
/// flame on whatever is burning, the same channel Earth's splash uses.
pub const BURN_VFX_BUFF: &str = "wan_fire_burn";
/// Held for as long as the burn actually lasts, not for one pass of the
/// animation — the tag loops, so this is what decides when it stops. Derived
/// from the burn itself so retuning the damage cannot desync the flame.
pub const BURN_VFX_TICKS: usize = BURN_TICKS * BURN_TICK_INTERVAL;

// -- skill2: Spirit Step --
/// The window itself: carries the movement speed and marks how long damage is
/// still being stored.
pub const STEP_BUFF: &str = "wan_spirit_step";
pub const STEP_MOVE_SPEED_PERCENT: i32 = 20;
pub const STEP_BUFF_DURATION: usize = 4 * 60;
/// Share of incoming damage banked during the window.
pub const STEP_STORE_PERCENT: usize = 80;
/// Share of the bank paid back as healing when the window ends.
pub const STEP_HEAL_PERCENT: usize = 80;
/// Paid on top of the bank, whether or not anything was banked — so casting it
/// and not being touched is still worth a little, rather than nothing.
pub const STEP_HEAL_FLAT: usize = 35;
/// Prefix of the bookkeeping buff. Its name carries the running total and the
/// last HP reading — see `match_hook`, which is what reads and rewrites it.
pub const STEP_STORE_PREFIX: &str = "wan_step_store";
/// The dash, mirrored from the `RushTime` effect in the .data_champion, which
/// is what actually performs it — these two exist only so `expected_move_distance`
/// can describe the skill to the AI. Keep them equal to `speed * tick` and
/// `tick` over there, or the AI will value a dash the champion does not have.
///
/// The `range` beside them over there must stay equal to this distance too. It
/// was 0, and he did not move at all: whether the engine reads that field as
/// how far the rush may carry him or as how far it looks for something to rush
/// at, zero is the answer that goes nowhere.
pub const STEP_DASH_DISTANCE: u64 = 60_000;
pub const STEP_DASH_TICKS: usize = 20;

// -- ult: Harmonic Convergence --
pub const CONVERGENCE_BUFF: &str = "wan_harmonic_convergence";
pub const CONVERGENCE_DURATION: usize = 8 * 60;
pub const CONVERGENCE_SHIELD: usize = 400;
pub const CONVERGENCE_SHIELD_AP_RATIO: usize = 80;
pub const CONVERGENCE_SHIELD_HP_RATIO: usize = 8;
/// Full strength, which is now the only strength: every proc is worth this,
/// in or out of the ult. The scale itself is kept because `element::proc` and
/// the burn's queued input still carry it end to end — that channel is what a
/// future "half strength" effect would ride on, and it costs nothing to leave
/// wired up.
pub const CONVERGENCE_BASE_SCALE: usize = 100;

/// Seconds to ticks.
#[allow(dead_code)]
pub fn ticks(seconds: f64) -> usize {
    (seconds * TICKS_PER_SECOND).round() as usize
}

/// `value * percent / 100`, rounded.
pub fn percent_of(value: usize, percent: usize) -> usize {
    (value * percent + 50) / 100
}

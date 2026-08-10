//! Small helpers shared by the effects.
//!
//! Everything here runs inside the deterministic simulation: same inputs must
//! produce the same result on every machine, so no clocks, no wall-time, and
//! no randomness except what is derived from the `rng_seed` a callback is
//! handed.

use mod_api_stable::*;

use crate::constants::MAP_SIZE;

/// Enemies of `caster_id` within `radius` of `center_id`, including
/// `center_id` itself. Collected before any mutation, since an entity view
/// borrows the sim and dealing damage needs it back.
pub fn enemies_near(
    sim: &StableSim<'_>,
    caster_id: usize,
    center_id: usize,
    radius: u64,
) -> Vec<usize> {
    let Some(caster) = sim.get_entity(caster_id) else {
        return Vec::new();
    };
    let team = caster.team();
    let radius_sq = radius.saturating_mul(radius);

    (0..sim.entity_count())
        .filter_map(|index| sim.entity_at(index))
        .filter(|entity| entity.is_alive() && entity.team() != team)
        .map(|entity| entity.id())
        .filter(|&id| id == center_id || sim.distance_sq(center_id, id) <= radius_sq)
        .collect()
}

/// The living enemy champion closest to `caster_id`.
///
/// This is what "forward" resolves to for Spirit Step. The stable API exposes
/// an entity's position but not its heading, so there is no facing to dash
/// along; the nearest enemy champion is the closest deterministic stand-in,
/// and it matches the skill's intent, since the bank only pays out if Wan
/// takes damage during the window.
///
/// `min_by_key` keeps the first of equal distances and entity order is
/// deterministic, so ties resolve the same way on every machine.
pub fn nearest_enemy_champion(sim: &StableSim<'_>, caster_id: usize) -> Option<usize> {
    let team = sim.get_entity(caster_id)?.team();

    (0..sim.entity_count())
        .filter_map(|index| sim.entity_at(index))
        .filter(|entity| entity.is_alive() && entity.is_champion() && entity.team() != team)
        .map(|entity| entity.id())
        .min_by_key(|&id| sim.distance_sq(caster_id, id))
}

/// The point `distance` world units from `from` toward `toward`, never
/// overshooting `toward` itself. Callers clamp the result with
/// [`clamp_to_map`].
///
/// Integer math throughout. Every machine has to land on the same coordinate
/// or the match desyncs, and floats are not worth that risk.
pub fn step_toward(from: (u64, u64), toward: (u64, u64), distance: u64) -> (u64, u64) {
    let dx = toward.0 as i64 - from.0 as i64;
    let dy = toward.1 as i64 - from.1 as i64;

    // Squares peak around 9.2e11 for a full-map delta, so i64 is comfortable.
    let length = isqrt((dx * dx + dy * dy) as u64) as i64;
    if length == 0 {
        return from;
    }

    let travel = (distance as i64).min(length);
    let x = from.0 as i64 + dx * travel / length;
    let y = from.1 as i64 + dy * travel / length;

    // `from` and `toward` are both on-map and `travel <= length`, so the
    // result cannot actually go negative — but a wrapped cast here would
    // clamp against the *far* edge later, so guard it rather than assume it.
    (x.max(0) as u64, y.max(0) as u64)
}

/// Holds a position inside the map, inset by `margin` on every side.
///
/// Pass the entity's radius as the margin: clamping the *centre* to the raw
/// bounds would leave half the body hanging off the edge. A margin wider than
/// the map collapses to the centre rather than inverting the range.
pub fn clamp_to_map(pos: (u64, u64), margin: u64) -> (u64, u64) {
    let margin = margin.min(MAP_SIZE / 2);
    (
        pos.0.clamp(margin, MAP_SIZE - margin),
        pos.1.clamp(margin, MAP_SIZE - margin),
    )
}

/// Integer square root by Newton's method. Hand-rolled rather than
/// `u64::isqrt` so the mod keeps building on older stable toolchains.
fn isqrt(value: u64) -> u64 {
    if value < 2 {
        return value;
    }

    let mut guess = value;
    let mut next = (guess + value / guess) / 2;
    while next < guess {
        guess = next;
        next = (guess + value / guess) / 2;
    }
    guess
}

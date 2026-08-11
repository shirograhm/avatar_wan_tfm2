//! Small helpers shared by the effects.
//!
//! Everything here runs inside the deterministic simulation: same inputs must
//! produce the same result on every machine, so no clocks, no wall-time, and
//! no randomness except what is derived from the `rng_seed` a callback is
//! handed.
//!
//! This used to carry a small geometry library — direction stepping, integer
//! square roots, map clamping, body collision — all of it in service of moving
//! Wan by hand for Spirit Step. The dash is a data-declared `RushTime` now, so
//! the engine does that and does it better; only the splash search is left.

use mod_api_stable::*;

/// Enemies of `caster_id` within `radius` of `center_id`, excluding
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
        .filter(|&id| id != center_id && sim.distance_sq(center_id, id) <= radius_sq)
        .collect()
}

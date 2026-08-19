use mod_api_stable::*;

use crate::constants::MAP_SIZE;

pub fn full_step_toward(
    from: (u64, u64),
    toward: (u64, u64),
    distance: u64,
    margin: u64,
) -> Option<((u64, u64), (i64, i64))> {
    let dx = toward.0 as i64 - from.0 as i64;
    let dy = toward.1 as i64 - from.1 as i64;

    let length = isqrt((dx * dx + dy * dy) as u64) as i64;
    if length == 0 {
        return None;
    }

    let step_x = dx * distance as i64 / length;
    let step_y = dy * distance as i64 / length;

    let margin = margin.min(MAP_SIZE / 2);
    let axis = |origin: u64, step: i64| {
        let inside = |step: i64| {
            let end = origin as i64 + step;
            (end >= margin as i64 && end <= (MAP_SIZE - margin) as i64)
                .then_some((end as u64, step))
        };
        inside(step).or_else(|| inside(-step)).unwrap_or_else(|| {
            let end = (origin as i64 + step).clamp(margin as i64, (MAP_SIZE - margin) as i64);
            (end as u64, end - origin as i64)
        })
    };

    let (x, step_x) = axis(from.0, step_x);
    let (y, step_y) = axis(from.1, step_y);
    Some(((x, y), (step_x, step_y)))
}

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

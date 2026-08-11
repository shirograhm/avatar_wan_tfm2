//! The four elements Wan cycles through, and the on-hit proc each one grants.
//!
//! The active element is stored as a named, statless buff on Wan's entity
//! rather than in Rust state. Effects receive `&self` and matches simulate in
//! parallel on several threads, so there is no sound place to hang per-entity
//! state on the mod side — but a buff is per-entity, deterministic, visible to
//! every callback, and cleaned up by the engine.

use mod_api_stable::*;

use crate::constants::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Air,
    Water,
    Earth,
    Fire,
}

/// The Avatar Cycle walks this ring in order. Wan starts on [`STARTING_ELEMENT`],
/// so his first cast moves him to the element after it.
pub const CYCLE: [Element; 4] = [Element::Air, Element::Water, Element::Earth, Element::Fire];

impl Element {
    pub fn buff_name(self) -> &'static str {
        match self {
            Element::Air => "wan_element_air",
            Element::Water => "wan_element_water",
            Element::Earth => "wan_element_earth",
            Element::Fire => "wan_element_fire",
        }
    }

    fn from_buff_name(name: &str) -> Option<Self> {
        CYCLE
            .into_iter()
            .find(|element| element.buff_name() == name)
    }

    pub fn next(self) -> Self {
        let index = CYCLE
            .iter()
            .position(|&element| element == self)
            .unwrap_or(0);
        CYCLE[(index + 1) % CYCLE.len()]
    }
}

/// The element Wan is currently in tune with, if any.
pub fn current(sim: &StableSim<'_>, entity: usize) -> Option<Element> {
    let entity = sim.get_entity(entity)?;
    (0..entity.buff_count())
        .filter_map(|index| entity.buff_at(index))
        .find_map(|buff| Element::from_buff_name(buff.name()))
}

/// Switches Wan to `element`, dropping whichever one he was on. Removing all
/// four rather than just the previous one keeps a single element attuned even
/// if something else ever applied one.
pub fn attune(sim: &mut StableSim<'_>, entity: usize, element: Element) {
    for stale in CYCLE {
        sim.entity_remove_buff(entity, stale.buff_name());
    }
    sim.add_buff(entity, &BuffV1::named(element.buff_name()));
}

/// How many stacks of `name` the entity is carrying.
pub fn buff_stacks(sim: &StableSim<'_>, entity: usize, name: &str) -> usize {
    let Some(entity) = sim.get_entity(entity) else {
        return 0;
    };
    (0..entity.buff_count())
        .filter_map(|index| entity.buff_at(index))
        .filter(|buff| buff.name() == name)
        .count()
}

pub fn has_buff(sim: &StableSim<'_>, entity: usize, name: &str) -> bool {
    buff_stacks(sim, entity, name) > 0
}

/// Fires one element's on-hit proc. `scale` is a percent — Harmonic
/// Convergence passes 150 for the element Wan is actually channeling and 100
/// for the other three.
pub fn proc(
    sim: &mut StableSim<'_>,
    caster: usize,
    target: usize,
    caster_stat: &StatV1,
    element: Element,
    scale: usize,
) {
    match element {
        // Stacking attack speed, capped. Each stack runs its own timer rather
        // than refreshing, since a buff layer cannot be extended once applied.
        Element::Air => {
            if buff_stacks(sim, caster, AIR_STACK_BUFF) >= AIR_MAX_STACKS {
                return;
            }
            let mut haste = BuffV1::timed(AIR_STACK_BUFF, AIR_DURATION);
            haste.attack_speed_mult = percent_of(AIR_ATTACK_SPEED_PERCENT as usize, scale) as i32;
            sim.add_buff(caster, &haste);
        }

        // A share of what he is missing, not a flat amount: worth nothing at
        // full health and most at the point he is about to die. Read from the
        // entity rather than `caster_stat`, which only carries max HP.
        Element::Water => {
            let Some((current, max)) = sim.get_entity(caster).map(|entity| entity.hp()) else {
                return;
            };
            let heal = percent_of(max.saturating_sub(current), WATER_MISSING_HP_PERCENT);
            sim.heal(caster, caster, percent_of(heal, scale));

            // Drawn on the target rather than on Wan, even though the heal
            // lands on him: it marks where the water actually struck, which is
            // what the projectile is pointing at. Layered like Earth's splash
            // and Fire's flame — a second hit inside the first's run time adds
            // its own copy on its own timer instead of restarting the burst.
            sim.add_buff(
                target,
                &BuffV1::timed(WATER_SPLASH_VFX_BUFF, WATER_SPLASH_VFX_TICKS),
            );
        }

        // Splashes off the struck target onto everything around it — the
        // target itself is spared, since the attack already hit it — so Earth
        // is the wave-clear element.
        Element::Earth => {
            let (physical, magic) = earth_splash_damage(caster_stat, scale);
            for splashed in crate::util::enemies_near(sim, caster, target, EARTH_RADIUS) {
                sim.deal_damage(caster, splashed, physical, magic, AttackTypeV1::BaseAttack);
            }

            // One burst, on the target the attack struck — it is the centre the
            // splash radiates from, so drawing a copy on every enemy caught in
            // it would read as several impacts instead of one. Note this is the
            // one enemy the splash does *not* damage.
            //
            // Layered on purpose: a second splash inside the first's run time
            // adds its own copy rather than restarting the animation, so rapid
            // hits read as overlapping impacts. Each stack runs its own timer,
            // the same way Air's do.
            sim.add_buff(
                target,
                &BuffV1::timed(EARTH_SPLASH_VFX_BUFF, EARTH_SPLASH_VFX_TICKS),
            );
        }

        // The burn is queued as repeating ticks of a registered native effect;
        // see `effects::FireBurnTick`. A queued effect carries no payload of
        // its own, so `scale` rides along in the input's unused `x` field —
        // both ends of that channel live in this mod.
        Element::Fire => {
            // Towers do not burn. A structure standing still under a damage
            // over time it cannot walk out of turns Fire into the siege
            // element by accident, which is Earth's job — and the flame drawn
            // on a tower reads as a bug besides.
            if sim.get_entity(target).is_some_and(|entity| entity.is_tower()) {
                return;
            }

            let input = InputTargetV1 {
                x: scale as u64,
                ..InputTargetV1::target(target)
            };
            for tick in 1..=BURN_TICKS {
                sim.queue_effect(
                    crate::effects::FIRE_BURN_TICK,
                    AttackTypeV1::Dot,
                    caster,
                    &input,
                    tick * BURN_TICK_INTERVAL,
                );
            }

            // Layered, like Earth's splash and like the burn itself: a second
            // application stacks its own flame on its own timer rather than
            // refreshing, so the fire is showing exactly while ticks are still
            // queued.
            sim.add_buff(target, &BuffV1::timed(BURN_VFX_BUFF, BURN_VFX_TICKS));
        }
    }
}

/// What each enemy caught in Earth's splash takes, as (physical, magic).
///
/// A share of the attack that produced it, split down the middle the same way
/// [`effects::SoulOfRaava`] splits the attack itself — so armour and magic
/// resist each answer only half of the splash, exactly as they do the hit.
pub fn earth_splash_damage(caster_stat: &StatV1, scale: usize) -> (usize, usize) {
    let total = percent_of(percent_of(caster_stat.attack, EARTH_ATTACK_SHARE), scale);
    (
        percent_of(total, ATTACK_PHYSICAL_SHARE),
        percent_of(total, ATTACK_MAGIC_SHARE),
    )
}

/// What one basic attack gains from the three elements Wan is *not* attuned
/// to — the offensive half of Harmonic Convergence, as (physical, magic).
///
/// Only Earth and Fire carry damage; Air and Water are worth real value but
/// not in a form `expected_damage` can express, so this understates the ult.
/// It also counts both Earth and Fire even though one of them may be the
/// element he is already on, which overstates it by one proc. Full strength
/// either way, since the ult no longer dilutes what it adds.
pub fn off_element_attack_damage(caster_stat: &StatV1) -> (usize, usize) {
    let (physical, earth_magic) = earth_splash_damage(caster_stat, CONVERGENCE_BASE_SCALE);
    let burn = burn_tick_damage(caster_stat, CONVERGENCE_BASE_SCALE) * BURN_TICKS;
    (physical, earth_magic + burn)
}

/// Damage of a single burn tick, recomputed from Wan's stats when it lands so
/// that items bought mid-burn are accounted for.
pub fn burn_tick_damage(caster_stat: &StatV1, scale: usize) -> usize {
    let total = BURN_DAMAGE
        + percent_of(caster_stat.attack, BURN_AD_RATIO)
        + percent_of(caster_stat.magic_power, BURN_AP_RATIO);
    percent_of(total / BURN_TICKS, scale)
}

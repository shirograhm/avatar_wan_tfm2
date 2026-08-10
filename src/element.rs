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

        Element::Water => {
            let heal = WATER_HEAL + percent_of(caster_stat.magic_power, WATER_AP_RATIO);
            sim.heal(caster, caster, percent_of(heal, scale));
        }

        // Splashes off the struck target rather than hitting it alone, so
        // Earth is the wave-clear element.
        Element::Earth => {
            let damage = percent_of(
                EARTH_DAMAGE + percent_of(caster_stat.magic_power, EARTH_AP_RATIO),
                scale,
            );
            for splashed in crate::util::enemies_near(sim, caster, target, EARTH_RADIUS) {
                sim.deal_damage(caster, splashed, damage, 0, AttackTypeV1::BaseAttack);
            }
        }

        // The burn is queued as repeating ticks of a registered native effect;
        // see `effects::FireBurnTick`. A queued effect carries no payload of
        // its own, so `scale` rides along in the input's unused `x` field —
        // both ends of that channel live in this mod.
        Element::Fire => {
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
        }
    }
}

/// Damage of a single burn tick, recomputed from Wan's stats when it lands so
/// that items bought mid-burn are accounted for.
pub fn burn_tick_damage(caster_stat: &StatV1, scale: usize) -> usize {
    let total = BURN_DAMAGE
        + percent_of(caster_stat.attack, BURN_AD_RATIO)
        + percent_of(caster_stat.magic_power, BURN_AP_RATIO);
    percent_of(total / BURN_TICKS, scale)
}

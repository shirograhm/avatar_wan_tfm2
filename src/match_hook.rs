use mod_api_stable::*;

use crate::constants::*;

pub struct WanDamageStore;

pub fn ledger_buff(banked: usize, last_hp: usize) -> BuffV1 {
    BuffV1::named(&format!("{STEP_STORE_PREFIX}|{banked}|{last_hp}"))
}

fn parse_ledger(name: &str) -> Option<(usize, usize)> {
    let rest = name.strip_prefix(STEP_STORE_PREFIX)?.strip_prefix('|')?;
    let (banked, last_hp) = rest.split_once('|')?;
    Some((banked.parse().ok()?, last_hp.parse().ok()?))
}

fn find_ledger(entity: &StableEntity<'_, '_>) -> Option<(String, usize, usize)> {
    (0..entity.buff_count())
        .filter_map(|i| entity.buff_at(i))
        .find_map(|buff| {
            let name = buff.name();
            parse_ledger(name).map(|(banked, hp)| (name.to_string(), banked, hp))
        })
}

fn window_open(entity: &StableEntity<'_, '_>) -> bool {
    (0..entity.buff_count()).any(|i| entity.buff_at(i).is_some_and(|b| b.name() == STEP_BUFF))
}

struct Pending {
    entity: usize,
    old_name: String,
    banked: usize,
    hp: usize,
    open: bool,
}

/// Wan is attuned from the moment he is on the field, so the element overlay
/// is never blank. Runs every tick rather than only at match start so a
/// respawn, which drops his buffs, re-attunes him too.
fn attune_unattuned(sim: &mut StableSim<'_>) {
    let unattuned: Vec<usize> = (0..sim.entity_count())
        .filter_map(|index| sim.entity_at(index))
        .filter(|entity| {
            entity.is_alive()
                && crate::element::is_wan(entity)
                && crate::element::current_of(entity).is_none()
        })
        .map(|entity| entity.id())
        .collect();

    for entity in unattuned {
        crate::element::attune(sim, entity, STARTING_ELEMENT);
    }
}

impl StableMatchHook for WanDamageStore {
    fn on_match_tick(&self, sim: &mut StableSim<'_>, _rng_seed: u64) {
        attune_unattuned(sim);

        let pending: Vec<Pending> = (0..sim.entity_count())
            .filter_map(|index| sim.entity_at(index))
            .filter(|entity| entity.is_alive())
            .filter_map(|entity| {
                let (old_name, banked, last_hp) = find_ledger(&entity)?;
                let hp = entity.hp().0;
                let taken = last_hp.saturating_sub(hp);
                Some(Pending {
                    entity: entity.id(),
                    old_name,
                    banked: banked + percent_of(taken, STEP_STORE_PERCENT),
                    hp,
                    open: window_open(&entity),
                })
            })
            .collect();

        for p in pending {
            sim.entity_remove_buff(p.entity, &p.old_name);

            if p.open {
                sim.add_buff(p.entity, &ledger_buff(p.banked, p.hp));
            } else {
                let heal = STEP_HEAL_FLAT + percent_of(p.banked, STEP_HEAL_PERCENT);
                if heal > 0 {
                    sim.heal(p.entity, p.entity, heal);
                }
            }
        }
    }
}

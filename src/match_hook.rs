//! Spirit Step's damage bank.
//!
//! Nothing in the stable API tells a data-declared champion how much damage it
//! just took: `StablePassive::on_damaged` only exists for a champion
//! registered from Rust, and Wan is declared in JSON. So the bank is measured
//! by sampling HP once per tick from a match hook, which runs every tick no
//! matter what state Wan is in.
//!
//! The running total has to live somewhere per-entity. It cannot live in this
//! struct: hook methods take `&self` and matches simulate in parallel across
//! threads, so mod-side state would be both unsound and shared between
//! matches. Instead it rides in the *name* of a bookkeeping buff —
//! `wan_step_store|<banked>|<last_hp>` — which is per-entity, deterministic,
//! and carries no stats of its own.

use mod_api_stable::*;

use crate::constants::*;

pub struct WanDamageStore;

/// Builds the bookkeeping buff for a given running total and HP reading.
pub fn ledger_buff(banked: usize, last_hp: usize) -> BuffV1 {
    BuffV1::named(&format!("{STEP_STORE_PREFIX}|{banked}|{last_hp}"))
}

/// Parses `wan_step_store|<banked>|<last_hp>` back out of a buff name.
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


/// One entity's ledger state for this tick.
struct Pending {
    entity: usize,
    old_name: String,
    banked: usize,
    hp: usize,
    open: bool,
}

impl StableMatchHook for WanDamageStore {
    fn on_match_tick(&self, sim: &mut StableSim<'_>, _rng_seed: u64) {
        // Read everything first: an entity view borrows the sim, and healing
        // and buff edits need it back.
        let pending: Vec<Pending> = (0..sim.entity_count())
            .filter_map(|index| sim.entity_at(index))
            .filter(|entity| entity.is_alive())
            .filter_map(|entity| {
                let (old_name, banked, last_hp) = find_ledger(&entity)?;
                let hp = entity.hp().0;
                // Only drops count. Healing mid-window must not refund the
                // bank, and `saturating_sub` also absorbs a max-HP change.
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
                // Window closed: pay the bank back and retire the ledger. The
                // flat share lands even on an empty bank, so a window nobody
                // punished still returns something.
                let heal = STEP_HEAL_FLAT + percent_of(p.banked, STEP_HEAL_PERCENT);
                if heal > 0 {
                    sim.heal(p.entity, p.entity, heal);
                }
            }
        }

    }
}

//! Effect logic — the part of an action that actually touches the simulation.
//!
//! `apply` runs inside the deterministic sim. The `expected_*` methods are
//! what the AI reads to decide whether casting is worth it; leaving them at
//! their defaults makes the champion read as harmless and it will never pick
//! the skill on purpose.

use mod_api_stable::*;

use crate::constants::*;
use crate::element::{self, Element, CYCLE};

// Registration names — the `effect_ref` values the .data_champion actions
// point at, except the burn tick, which `element::proc` queues at runtime.
pub const AVATAR_CYCLE: &str = "avatar_wan_avatar_cycle";
pub const SPIRIT_STEP: &str = "avatar_wan_spirit_step";
pub const HARMONIC_CONVERGENCE: &str = "avatar_wan_harmonic_convergence";
pub const FIRE_BURN_TICK: &str = "avatar_wan_fire_burn_tick";

fn stat_of(sim: &StableSim<'_>, entity: usize) -> StatV1 {
    sim.get_entity(entity)
        .map_or(StatV1::default(), |entity| entity.stat())
}

// ------------------------------------------------ basic attack: Soul of Raava

/// Wan's basic attack, applied when his projectile lands: his attack damage
/// split half physical / half magic, plus the on-hit proc of the element he
/// was attuned to **when he fired**.
///
/// The element is baked into the effect rather than read at impact. The
/// `.data_champion` branches on the element buff at launch to pick the
/// projectile visual, and each branch points at the matching one of these —
/// so the shot that leaves as fire lands as fire even if Wan cycles
/// mid-flight, and the visual can never disagree with the proc.
pub struct SoulOfRaava(pub Element);

/// The four registered names, paired with the element each one procs. The
/// `.data_champion` picks between them in its `SwitchByBuff` chain.
pub const ATTACK_HITS: [(&str, Element); 4] = [
    ("avatar_wan_attack_hit_air", Element::Air),
    ("avatar_wan_attack_hit_water", Element::Water),
    ("avatar_wan_attack_hit_earth", Element::Earth),
    ("avatar_wan_attack_hit_fire", Element::Fire),
];

impl StableEffectType for SoulOfRaava {
    fn apply(
        &self,
        sim: &mut StableSim<'_>,
        _rng_seed: u64,
        caster_id: usize,
        input: InputTargetV1,
    ) {
        if InputTargetKindV1::from_code(input.kind) != Some(InputTargetKindV1::Target) {
            return;
        }
        let target = input.target_id;

        // Every read happens before the first mutation: an entity view borrows
        // the sim, and dealing damage needs it back.
        let stat = stat_of(sim, caster_id);
        let converged = element::has_buff(sim, caster_id, CONVERGENCE_BUFF);
        let launched = self.0;

        // Bootstrap the attunement on the first hit of a life. There is no
        // passive to do it now that the champion is data-declared, and the
        // buff is what the cycle and the projectile branch both read.
        if element::current(sim, caster_id).is_none() {
            element::attune(sim, caster_id, launched);
        }

        // Full attack damage, split half physical / half magic so armour and
        // magic resist each only answer one side of it.
        sim.deal_damage(
            caster_id,
            target,
            percent_of(stat.attack, ATTACK_PHYSICAL_SHARE),
            percent_of(stat.attack, ATTACK_MAGIC_SHARE),
            AttackTypeV1::BaseAttack,
        );

        // Under the ult the attuned element is unchanged and the other three
        // come along at half strength — the ult is breadth, not a multiplier.
        if converged {
            for candidate in CYCLE {
                let scale = if candidate == launched {
                    CONVERGENCE_BASE_SCALE
                } else {
                    CONVERGENCE_OFF_ELEMENT_SCALE
                };
                element::proc(sim, caster_id, target, &stat, candidate, scale);
            }
        } else {
            element::proc(
                sim,
                caster_id,
                target,
                &stat,
                launched,
                CONVERGENCE_BASE_SCALE,
            );
        }
    }

    fn expected_damage(&self, caster_stat: &StatV1) -> (usize, usize) {
        // Split the same way it is dealt, so the AI weighs him against both
        // resistances rather than one.
        (
            percent_of(caster_stat.attack, ATTACK_PHYSICAL_SHARE),
            percent_of(caster_stat.attack, ATTACK_MAGIC_SHARE),
        )
    }
}

/// One tick of the fire burn, queued six times by [`element::proc`]. The
/// scale rides in the input's `x` field; a zero there means an unscaled tick.
pub struct FireBurnTick;

impl StableEffectType for FireBurnTick {
    fn apply(
        &self,
        sim: &mut StableSim<'_>,
        _rng_seed: u64,
        caster_id: usize,
        input: InputTargetV1,
    ) {
        let scale = if input.x == 0 {
            CONVERGENCE_BASE_SCALE
        } else {
            input.x as usize
        };
        let stat = stat_of(sim, caster_id);
        let damage = element::burn_tick_damage(&stat, scale);
        sim.deal_damage(caster_id, input.target_id, 0, damage, AttackTypeV1::Dot);
    }

    fn expected_damage(&self, caster_stat: &StatV1) -> (usize, usize) {
        (
            0,
            element::burn_tick_damage(caster_stat, CONVERGENCE_BASE_SCALE),
        )
    }
}

// ------------------------------------------------------- skill: The Avatar Cycle

/// Swaps Wan to the next element in the cycle.
pub struct TheAvatarCycle;

impl StableEffectType for TheAvatarCycle {
    fn apply(
        &self,
        sim: &mut StableSim<'_>,
        _rng_seed: u64,
        caster_id: usize,
        _input: InputTargetV1,
    ) {
        // Cycling before the first attack still advances *past* the starting
        // element, so the sequence reads the same either way.
        let next = element::current(sim, caster_id)
            .unwrap_or(STARTING_ELEMENT)
            .next();
        element::attune(sim, caster_id, next);
    }

    fn on_caster(&self) -> bool {
        true
    }

    fn can_move(&self) -> bool {
        true
    }
}

// ------------------------------------------------------- skill2: Spirit Step

/// Opens the damage-banking window: movement speed now, healing later.
///
/// This effect only starts the window. The banking and the payout live in
/// `match_hook::WanDamageStore`, because nothing reports damage taken to a
/// data-declared champion — `StablePassive::on_damaged` only exists for a
/// champion registered from Rust, and Wan is declared in JSON. The hook
/// samples HP every tick instead.
pub struct SpiritStep;

impl SpiritStep {
    fn buff() -> BuffV1 {
        let mut buff = BuffV1::timed(STEP_BUFF, STEP_BUFF_DURATION);
        buff.move_speed_mult = STEP_MOVE_SPEED_PERCENT;
        buff
    }
}

impl StableEffectType for SpiritStep {
    fn apply(
        &self,
        sim: &mut StableSim<'_>,
        _rng_seed: u64,
        caster_id: usize,
        _input: InputTargetV1,
    ) {
        // Seed the ledger with his current HP, so the hook's first sample
        // measures damage from the instant of the cast rather than from zero.
        let hp = sim.get_entity(caster_id).map_or(0, |entity| entity.hp().0);

        sim.add_buff(caster_id, &Self::buff());
        sim.add_buff(caster_id, &crate::match_hook::ledger_buff(0, hp));
    }

    fn expected_buff(&self, _caster_stat: &StatV1) -> Option<BuffV1> {
        Some(Self::buff())
    }

    /// The AI reads this to value the skill. The real payout depends on damage
    /// taken during the window, which is unknowable at cast time, so quote the
    /// window's own length as a rough stand-in for its worth.
    fn expected_heal(&self, caster_stat: &StatV1) -> usize {
        percent_of(
            percent_of(caster_stat.hp, STEP_STORE_PERCENT),
            STEP_HEAL_PERCENT,
        ) / 4
    }

    fn on_caster(&self) -> bool {
        true
    }

    fn can_move(&self) -> bool {
        true
    }
}

// ------------------------------------------------- ult: Harmonic Convergence

/// Eight seconds of every element at once, behind a shield.
pub struct HarmonicConvergence;

impl HarmonicConvergence {
    /// Statless on purpose: the buff exists so `SoulOfRaava` can tell the ult
    /// is running. The shield is applied separately, as its own layer.
    fn buff() -> BuffV1 {
        BuffV1::timed(CONVERGENCE_BUFF, CONVERGENCE_DURATION)
    }

    /// `StatV1::hp` is the caster's max HP including items and buffs, so the
    /// health share grows with what he actually built.
    fn shield_amount(caster_stat: &StatV1) -> usize {
        CONVERGENCE_SHIELD
            + percent_of(caster_stat.magic_power, CONVERGENCE_SHIELD_AP_RATIO)
            + percent_of(caster_stat.hp, CONVERGENCE_SHIELD_HP_RATIO)
    }
}

impl StableEffectType for HarmonicConvergence {
    fn apply(
        &self,
        sim: &mut StableSim<'_>,
        _rng_seed: u64,
        caster_id: usize,
        _input: InputTargetV1,
    ) {
        let stat = stat_of(sim, caster_id);

        sim.add_buff(caster_id, &Self::buff());
        sim.entity_add_shield(caster_id, Self::shield_amount(&stat), CONVERGENCE_DURATION);
    }

    fn expected_buff(&self, _caster_stat: &StatV1) -> Option<BuffV1> {
        Some(Self::buff())
    }

    fn expected_shield(&self, caster_stat: &StatV1) -> usize {
        Self::shield_amount(caster_stat)
    }

    fn on_caster(&self) -> bool {
        true
    }

    fn can_move(&self) -> bool {
        true
    }
}

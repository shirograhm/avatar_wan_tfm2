//! Makes Wan commit to fights the default AI would walk away from.
//!
//! The default input AI is generic: it disengages on a health threshold that
//! assumes damage taken is simply lost. Two thirds of Wan's kit says otherwise
//! — Spirit Step pays back 80% of what he takes, Harmonic Convergence eats it
//! behind a shield, and Water heals more the lower he is. Retreating through
//! those windows is the one thing that makes them worthless.
//!
//! So this narrows when he is allowed to break off, rather than driving him
//! outright: it returns `None` — keep whatever the default decided — for every
//! case it has no opinion about, and only substitutes an attack when he is
//! healthy enough to trade and something is close enough to trade with.
//!
//! Outside his own windows it will only overrule a *move*, so his target
//! selection and his kiting are left to the default AI. Inside one, it will
//! also overrule a retreat or a recall, and it will walk him half a range
//! further to find someone to spend the window on.
//!
//! Harmonic Convergence is the hard case: it is worth only what he attacks
//! into it, so while it runs he attacks and does nothing else — a longer walk
//! to reach a champion, an Avatar Cycle cast overruled (all four elements are
//! already firing), and anything at all in range taken as a target rather than
//! spending the window standing still. His health floor drops to
//! [`AGGRO_ULT_HP_FLOOR`] for the duration, low enough that only actually
//! dying pulls him out.
//!
//! It also vetoes one cast: Spirit Step, when he is hurt and alone. The skill
//! is judged by `expected_heal`, which is a static number — it cannot know
//! whether anyone is around to deal the damage it pays back. Here that is
//! knowable, so the check lives here rather than in the effect.
//!
//! `think` is a pure decision: read the sim, return an input, mutate nothing.

use mod_api_stable::*;

use crate::constants::*;
use crate::element;

/// What `think` decided while it held the sim, resolved into an input after
/// it let go.
enum Plan {
    /// Keep whatever the default AI chose.
    Keep,
    Attack(usize),
    /// Break off instead of spending Spirit Step on an empty window.
    Disengage,
}

pub struct AggressiveWan;

impl AggressiveWan {
    /// True while a window is running that turns damage taken into value.
    fn committed(sim: &StableSim<'_>, entity: usize) -> bool {
        element::has_buff(sim, entity, STEP_BUFF)
            || element::has_buff(sim, entity, CONVERGENCE_BUFF)
    }

    /// True while Harmonic Convergence is running — the window whose whole
    /// value is in the attacks he lands during it.
    fn converging(sim: &StableSim<'_>, entity: usize) -> bool {
        element::has_buff(sim, entity, CONVERGENCE_BUFF)
    }

    /// Nearest living enemy of *any* kind within `range` — creeps and towers
    /// included, unlike [`Self::enemy_champion_near`].
    ///
    /// Only the ult uses this, and only at attack range, so it never sends him
    /// walking at a creep: it is the difference between finishing the window
    /// swinging at whatever is in front of him and standing still.
    fn any_enemy_near(sim: &StableSim<'_>, entity: usize, range: u64) -> Option<usize> {
        crate::util::enemies_near(sim, entity, entity, range)
            .into_iter()
            // Ties resolve by entity order, which is deterministic.
            .min_by_key(|&id| sim.distance_sq(entity, id))
    }

    /// Nearest living enemy champion within `range` of `entity`.
    ///
    /// Champions only — creeps and towers are not what he is being kept in the
    /// fight for, and chasing one under a tower is how "aggressive" turns into
    /// "throws the game".
    fn enemy_champion_near(sim: &StableSim<'_>, entity: usize, range: u64) -> Option<usize> {
        let team = sim.get_entity(entity)?.team();
        let range_sq = range.saturating_mul(range);

        (0..sim.champion_count())
            .map(|index| sim.champion_id_at(index))
            .filter(|&id| {
                sim.get_entity(id)
                    .is_some_and(|other| other.is_alive() && other.team() != team)
            })
            .map(|id| (sim.distance_sq(entity, id), id))
            .filter(|&(distance_sq, _)| distance_sq <= range_sq)
            // Ties resolve by champion order, which is deterministic.
            .min_by_key(|&(distance_sq, _)| distance_sq)
            .map(|(_, id)| id)
    }
}

impl StablePlayerAi for AggressiveWan {
    fn clone_box(&self) -> Box<dyn StablePlayerAi> {
        Box::new(AggressiveWan)
    }

    fn id(&self) -> String {
        "avatar_wan_aggression".to_string()
    }

    fn matches(&self, init: &StableAiInit) -> bool {
        init.champion_name == CHAMPION_ID
    }

    fn think(&mut self, ctx: &mut StableAiContext<'_>, base_input: Option<InputV1>) -> Option<InputV1> {
        let base_kind = base_input.and_then(|input| InputKindV1::from_code(input.kind));

        // Anything other than these is a cast he already decided on and is not
        // ours to second-guess — except Spirit Step, handled below. An unknown
        // kind lands here too: something newer than this build is not ours to
        // touch either.
        if !matches!(
            base_kind,
            Some(InputKindV1::Move)
                | Some(InputKindV1::Return)
                | Some(InputKindV1::Attack)
                | Some(InputKindV1::Skill)
                | Some(InputKindV1::Skill2)
                | None
        ) {
            return None;
        }

        let player = ctx.player_id();
        let hp_percent = ctx.hp_ratio_percent()?;

        // Decided while the sim is borrowed, acted on once it is released —
        // the input helpers below need the context back.
        let plan = {
            let sim = ctx.sim()?;
            let me = sim.get_player(player).and_then(|player| player.champion())?;
            let me = me.id();

            let committed = Self::committed(&sim, me);
            let converging = Self::converging(&sim, me);
            let in_combat = Self::enemy_champion_near(&sim, me, AGGRO_COMBAT_RANGE).is_some();

            if base_kind == Some(InputKindV1::Skill2) {
                // Spirit Step pays out a share of the damage taken during its
                // window. Cast with nobody in range to deal any, it banks
                // nothing and heals nothing — the AI reads its `expected_heal`
                // and treats it as a heal to press when hurt, which is exactly
                // wrong when he is hurt *and alone*. That hint cannot know the
                // difference; it has no sim. This can.
                if !in_combat && hp_percent < AGGRO_LOW_HP {
                    Plan::Disengage
                } else {
                    Plan::Keep
                }
            } else if base_kind == Some(InputKindV1::Skill) && !converging {
                // Which element he is on is his call — the cycle is the one
                // decision the default AI makes with information this does not
                // have. The single exception is handled just below.
                Plan::Keep
            } else if converging && hp_percent < AGGRO_ULT_HP_FLOOR {
                // The one exit from the window: this low, the shield is spent
                // and the next hit kills him, which ends the ult far more
                // completely than walking out of it does.
                Plan::Disengage
            } else if converging && base_kind == Some(InputKindV1::Attack) {
                // Already swinging, which is the whole point — and the default
                // AI's target knows things about focus fire that "nearest"
                // does not.
                Plan::Keep
            } else if converging {
                // Harmonic Convergence is worth exactly as many attacks as he
                // lands inside it: every attack procs all four elements, and
                // the shield is there so that standing in range costs him
                // nothing. So for these eight seconds nothing else is worth
                // doing — not retreating, not recalling, and not cycling,
                // since all four elements are already firing at full strength
                // and the cast would only cost him a swing.
                //
                // The health floor is [`AGGRO_ULT_HP_FLOOR`] rather than the
                // committed one, and it is checked above: the shield is what
                // breaks first, so backing off with it still up wastes the
                // window it was spent on.
                match Self::enemy_champion_near(&sim, me, AGGRO_ULT_ENGAGE_RANGE)
                    // Nobody worth walking at — swing at whatever is already
                    // in front of him instead of idling out the window.
                    .or_else(|| Self::any_enemy_near(&sim, me, ATTACK_RANGE))
                {
                    Some(target) => Plan::Attack(target),
                    None => Plan::Keep,
                }
            } else if !committed && base_kind != Some(InputKindV1::Move) {
                // Outside a window he only gets talked out of *walking away* —
                // his own attacks keep the default AI's target, which knows
                // things this does not about focus fire, and his kiting stays
                // intact. Recalling also stands: he wants to shop and heal, and
                // only a running window makes leaving strictly worse.
                Plan::Keep
            } else {
                let floor = if committed {
                    AGGRO_COMMITTED_HP_FLOOR
                } else {
                    AGGRO_HP_FLOOR
                };

                // He will walk into a fight during a window; outside one he
                // only holds ground he is already standing on.
                let reach = if committed {
                    AGGRO_ENGAGE_RANGE
                } else {
                    ATTACK_RANGE
                };

                match Self::enemy_champion_near(&sim, me, reach) {
                    Some(target) if hp_percent >= floor => Plan::Attack(target),
                    _ => Plan::Keep,
                }
            }
        };

        match plan {
            Plan::Keep => None,
            Plan::Attack(target) => {
                let input = InputV1::action(InputKindV1::Attack, InputTargetV1::target(target));
                ctx.is_valid_input(&input).then_some(input)
            }
            // Going back for a real heal beats a skill that cannot pay out.
            // `run_away_without_skill_input` is the fallback precisely because
            // it will not spend the cooldown we are trying to save.
            Plan::Disengage => {
                if ctx.is_safe_to_recall() {
                    ctx.recall_input()
                } else {
                    ctx.run_away_without_skill_input()
                }
            }
        }
    }
}

//! Avatar Wan — a stable-ABI champion mod for Teamfight Manager 2.
//!
//! Wan himself is declared in `champion/avatar_wan.data_champion`: stats,
//! growth, sprite, icons, and the four action shells. Each of those actions
//! points back here with `{"type":"Native","effect_ref":"…"}`, so this DLL
//! supplies only the logic — which is the part the data format cannot express,
//! since Wan's kit is stateful (element attunement) rather than a fixed
//! sequence of built-in effects.
//!
//! Two rules keep this DLL loading across game updates:
//!   1. Ignore unknown enum codes — anything newer than this build arrives as
//!      `None` and must be skipped, not guessed at.
//!   2. Write failure-tolerant code — on an older game, calls into newer
//!      features return `None`/`false`/empty rather than crashing.
//!
//! A panic inside any callback disables the mod but leaves the game running,
//! so prefer returning early over unwrapping.

use mod_api_stable::*;

mod constants;
mod effects;
mod element;
mod match_hook;
mod util;

use constants::MOD_ID;

fn init(host: &StableHost) -> StableMod {
    host.log(
        LogLevel::Info,
        "avatar_wan_tfm2: registering Avatar Wan effects",
    );

    let mut reg = StableMod::new(MOD_ID);

    // Every name here must match an `effect_ref` in the .data_champion —
    // except the burn tick, which nothing references statically because
    // `element::proc` queues it at runtime.
    for (name, element) in effects::ATTACK_HITS {
        reg.add_native_effect(name, effects::SoulOfRaava(element));
    }
    reg.add_native_effect(effects::AVATAR_CYCLE, effects::TheAvatarCycle);
    reg.add_native_effect(effects::SPIRIT_STEP, effects::SpiritStep);
    reg.add_native_effect(effects::HARMONIC_CONVERGENCE, effects::HarmonicConvergence);
    reg.add_native_effect(effects::FIRE_BURN_TICK, effects::FireBurnTick);

    // Spirit Step banks damage taken across its window. That has to be
    // measured from outside the skill — nothing reports damage to a
    // data-declared champion — so a match hook samples HP every tick.
    reg.set_match_hook(match_hook::WanDamageStore);

    reg
}

declare_stable_mod!(init);

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

    for (name, element) in effects::ATTACK_HITS {
        reg.add_native_effect(name, effects::SoulOfRaava(element));
    }
    reg.add_native_effect(effects::AVATAR_CYCLE, effects::TheAvatarCycle);
    reg.add_native_effect(effects::SPIRIT_STEP, effects::SpiritStep);
    reg.add_native_effect(effects::HARMONIC_CONVERGENCE, effects::HarmonicConvergence);
    reg.add_native_effect(effects::FIRE_BURN_TICK, effects::FireBurnTick);

    reg.set_match_hook(match_hook::WanDamageStore);

    reg
}

declare_stable_mod!(init);

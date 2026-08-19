use mod_api_stable::*;

mod constants;
mod effects;
mod element;
mod match_hook;
mod player_ai;
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

    // Keeps him in fights his kit is built to win. `matches` limits it to
    // Wan's own athletes, so no other champion's AI is touched.
    reg.add_player_input_ai(player_ai::AggressiveWan);

    reg
}

declare_stable_mod!(init);

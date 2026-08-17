# Avatar Wan

Adds Wan, the first Avatar, as a playable champion. Built against the
**stable ABI** (`mod-api-stable`), so the DLL keeps loading across game
updates and builds with any Rust toolchain — no pinned nightly.

The champion itself is declared in `champion/avatar_wan.data_champion`; the
Rust side only registers the native effects that data can't express, plus a
per-tick match hook. Art is complete — sprite, skill icons, projectiles, and
per-element view effects all ship.

## Kit

- **Basic attack — Soul of Raava.** Deals 50% AD physical + 50% AD magic, and
  procs the element Wan is attuned to.
- **Skill (Lv. 1) — The Avatar Cycle.** Swap to the next element in the cycle
  (Air → Water → Earth → Fire → Air). Wan spawns on Fire.
  - **Fire:** burn for 12 (+30% AP) magic damage over 3s, in 6 ticks 30 ticks
    apart. Towers do not burn.
  - **Air:** +6% Move Speed for 4s on hit, up to 4 stacks.
  - **Water:** heals 3 + 3% of missing HP on hit.
  - **Earth:** 40% of the attack's damage, split physical/magic the same way,
    splashed to enemies within 40,000 units of the target — the struck target
    itself excluded (the attack already hit it), and towers excluded.
- **Skill 2 (Lv. 3) — Spirit Step.** Dash 60,000 units, then for 3s bank 80% of
  damage taken and gain +10% (+0.03% AD) Attack Speed; when it expires, heal
  for 30 + 80% of the banked amount.
- **Ult — Harmonic Convergence.** 6s behind a 300 (+60% AP) (+6% HP) shield;
  basic attacks proc all four elements. Each takedown (kill *or* assist) adds
  1.5s.

Every number lives in `src/constants.rs`. Anything user-facing is duplicated by
hand in `text/champion.i18n` — the descriptions are static strings, so changing
a constant means editing the i18n too.

## Layout

| Path | What it is |
| --- | --- |
| `champion/avatar_wan.data_champion` | The champion: stats, growth, the four action slots, and every `view_effect` / `view_buff` / `view_projectile` |
| `src/lib.rs` | Entry point — `declare_stable_mod!(init)`, registers the five native effects and the match hook |
| `src/effects.rs` | The four slots' native effects + the burn tick, and their `expected_*` AI valuations |
| `src/element.rs` | The four elements, attunement, and each element's on-hit proc |
| `src/match_hook.rs` | Per-tick work: Spirit Step's damage ledger, ult extension on takedown, re-attuning on spawn/respawn |
| `src/constants.rs` | Every tunable number, in world units and ticks |
| `src/util.rs` | Radius query for Earth's splash |
| `aseprite_resources/champions/` | Battle sprite (`avatar_wan#sheet.png` + `#anim.fanim`) |
| `icons/` | Skill icons |
| `effects/` | Per-element and per-skill VFX sheets |
| `style/champion_view.champion_view` | Face/center anchor offsets for the portrait |
| `text/champion.i18n` | Name + skill descriptions, merged into `asset/base/text/champion` |
| `mod.override_info` | The merge rules for the text and style files |
| `deploy.ps1` | Build + wipe + redeploy into the game's `mods/` folder |
| `tools/`, `art_src/` | Python helpers for generating sheets. Not deployed. |

## Implementation notes

**The active element is a buff, not Rust state.** The four slots are separate
objects with no shared state — effects receive `&self`, and matches simulate in
parallel across threads, so there is nowhere sound to hang per-entity state
mod-side. A stateless named buff (`wan_element_fire`, …) is per-entity,
deterministic, readable from every callback, and cleared by the engine on
death. `element::attune` removes all four before adding one, so he can never
end up on two.

**The basic attack branches in data, not in Rust.** `avatar_wan.data_champion`
nests `SwitchByBuff` five deep — convergence, then air/water/earth, falling
through to fire — so each element gets its own projectile art and its own
native hit effect (`avatar_wan_attack_hit_*`). That's also what makes the AI's
damage estimate honest: `expected_damage` on each `SoulOfRaava(element)`
instance prices exactly the shot that branch will fire.

**The fire burn is queued, not a buff.** The stable API has no damage-over-time
primitive, so the burn queues six ticks of a registered native effect
(`avatar_wan_fire_burn_tick`) at 30-tick intervals. Each tick recomputes its
damage from Wan's *current* stats, so items bought mid-burn count. Re-applying
the burn stacks new ticks on top of the old ones rather than refreshing —
change `element::proc` if you want refresh behaviour instead.

**Spirit Step's banked damage is a buff name.** Buffs carry no payload, so the
ledger is encoded into the name itself: `wan_step_store|<banked>|<last_hp>`.
`match_hook` reads it every tick, diffs HP against `last_hp`, adds 80% of the
drop to `banked`, and rewrites the buff. When the `wan_spirit_step` window buff
falls off, the ledger pays out as a heal and is removed.

**Takedown extension is a remove-and-re-add.** `BuffV1` has no "extend"
operation, so `extend_convergence_on_takedown` reads the remaining duration off
the live buff, removes it, and re-adds it with the bonus rolled in. The shield
rides the same clock and is rebuilt the same way — `entity_clear_shield` is not
layer-addressable, so this also sweeps up any other shield Wan is carrying. In
practice the ult's is the only one.

**Takedowns are read one tick late.** `takedowns_last_tick` filters the kill log
for `tick == sim.tick() - 1`, so each kill is counted exactly once regardless of
whether the host appends to the log before or after the match hook runs.

**Attunement is re-applied every tick, not at match start.** `attune_unattuned`
sweeps for a living Wan with no element buff. That covers respawns, which drop
his buffs, and keeps the element overlay from ever going blank.

**Some AI valuations are deliberate guesses.** Neither `expected_damage` nor
`expected_heal` can read current HP, so `EXPECTED_MISSING_HP_PERCENT` (25%)
stands in when pricing Water's heal, and Spirit Step's `expected_heal` assumes
he banks a quarter of his max HP. Both are tuning knobs, not measurements.

## Build

```sh
cargo build --release
cp target/release/avatar_wan_tfm2.dll ./avatar_wan_tfm2.dll
```

The DLL filename must match the mod folder name, and the id passed to
`StableMod::new` must match it too.

Prefer `.\deploy.ps1` (also bound to F5), which builds, copies the DLL to the
repo root, wipes the deployed folder, and lays down only the files the engine
reads. The wipe is deliberate: a file deleted from the repo would otherwise
linger in the game folder and keep being loaded. It refuses to run while the
game is open, and refuses any target that isn't a `mods\avatar_wan_tfm2`
folder. Point it elsewhere with `-GameDir`; it defaults to
`D:\SteamLibrary\steamapps\common\Teamfight Manager2`.

## Known loose ends

- `icons/avatar_wan_base_attack.png` exists but nothing references it — the
  `skill_icons` array in the `.data_champion` only lists the three skills.
  Confirm whether the engine wants a fourth entry for the basic attack or
  sources that icon elsewhere.
- `aseprite_resources/inquisitor#anim.fanim` is a stray from another champion
  and gets deployed with the rest of the folder. Harmless, but it can go.
- No ban/pick illustration yet. `banpick_illustrations/avatar_wan.png`
  (512 × 640 sRGB) would replace the pixel-sprite fallback — note the docs
  describe these as presentation-only packs registered with
  `"mod_type": "banpick_illustration"`, so it's worth confirming that can sit
  on a champion mod rather than wanting its own folder.
- No `thumbnail.png` / `preview.png` in the mod root for the Workshop listing.
  `previews/` has raw material but nothing sized for it.
- `text/champion.i18n` is English only.

## Notes on the simulation

Every gameplay callback runs inside the deterministic sim: same inputs must
give the same result on every machine. Derive randomness only from the
`rng_seed` you are handed — never from clocks, addresses, or global state.

Distances are world units (960,000 × 960,000 map; champion radius ≈ 10,000;
melee range ≈ 12,000) and durations are ticks (60 = 1 second).

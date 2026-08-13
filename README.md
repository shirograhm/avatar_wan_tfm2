# Avatar Wan

Adds Wan, the first Avatar, as a playable champion. Built against the
**stable ABI** (`mod-api-stable`), so the DLL keeps loading across game
updates and builds with any Rust toolchain — no pinned nightly.

## Kit

- **Basic attack — Soul of Raava.** Deals 100% AD, split half magic and half
  physical, and procs the element Wan is attuned to.
- **Skill (Lv. 1) — The Avatar Cycle.** Swap to the next element in the cycle
  (Air → Water → Earth → Fire → Air). Wan spawns on Fire.
  - **Fire:** burn for 10 (+15% AD) (+35% AP) magic damage over 3s.
  - **Air:** +4% Attack Speed for 2s on hit, up to 4 stacks.
  - **Water:** heals 5 (+5% AP) on hit.
  - **Earth:** 50% of the attack's own damage, split physical/magic the same
    way, splashed to enemies within 40,000 units of the target — the struck
    target itself excluded, since the attack already hit it.
- **Skill 2 (Lv. 3) — Spirit Step.** Dash forward 60,000 units, then for 2s
  bank 80% of damage taken and gain +20% Movement Speed; when it expires,
  heal for 35 + 80% of the banked amount.
- **Ult — Harmonic Convergence.** 8s behind a 400 (+80% AP) (+8% HP) shield;
  basic attacks proc all four elements at full strength.

## Layout

| Path | What it is |
| --- | --- |
| `src/lib.rs` | Entry point — `declare_stable_mod!(init)`, registers the champion and the named burn effect |
| `src/champion.rs` | Identity, stats, growth, action slots, lane prior |
| `src/actions.rs` | Animation/cooldown/targeting shell for each of the four slots |
| `src/effects.rs` | The logic that touches the simulation |
| `src/element.rs` | The four elements, attunement, and each element's on-hit proc |
| `src/passive.rs` | Sets the starting attunement on spawn |
| `src/constants.rs` | Every tunable number, in world units and ticks |
| `src/util.rs` | Dash direction resolution |
| `text/champion.i18n` | Name + skill descriptions, merged into `asset/base/text/champion` |
| `mod.override_info` | The merge rule for the above |

## Three implementation notes

**The active element is a buff, not Rust state.** Soul of Raava, The Avatar Cycle
and Harmonic Convergence are separate objects with no shared state — effects
receive `&self`, and matches simulate in parallel across threads, so there is
nowhere sound to hang per-entity state mod-side. A statless named buff
(`wan_element_fire`, …) is per-entity, deterministic, readable from every
callback, and cleared by the engine on death. `element::attune` removes all
four before adding one, so he can never end up on two.

**The fire burn is queued, not a buff.** The stable API has no damage-over-time
primitive, so the burn queues four ticks of a registered native effect
(`avatar_wan_fire_burn_tick`) at 30-tick intervals. Each tick recomputes its
damage from Wan's *current* stats, so items bought mid-burn count. Re-applying
the burn stacks new ticks on top of the old ones rather than refreshing —
change `element::proc` if you want refresh behaviour instead.

**Harmonic Convergence's 150% rides in the queued input.** A queued effect
carries no payload, so the scale is stashed in the input's unused `x` field.
Both ends of that channel are in this mod; a `0` there reads as 100%.

## Build

```sh
cargo build --release
cp target/release/avatar_wan_tfm2.dll ./avatar_wan_tfm2.dll
```

The DLL filename must match the mod folder name, and the id passed to
`StableMod::new` must match it too. Then copy the whole folder into the
game's `mods/` directory.

`.vscode/launch.json` currently points at `mod-sdk-0.5.2/build_mod_cargo.ps1`.
That script is for legacy mods — it forces the SDK's pinned nightly and links
the prebuilt `mod_api` rlib, neither of which a stable-ABI mod needs. Plain
`cargo build --release` is the whole build.

## Still to do (art)

The code side is complete and compiles; the champion has no art yet.

A champion registered from Rust gets its visuals **asset-driven by its id** —
the same rules as a data champion. Nothing about art is declared in the Rust
code (except the skill-icon sheet, which `champion.rs::skill_icon` overrides);
you drop files at the id-derived paths and the engine finds them.

### 1. Battle sprite — required

```
champion/avatar_wan.aseprite          -> asset/avatar_wan_tfm2/champion/avatar_wan
```

Aseprite user data must declare the sheet type, which is what produces the
`#sheet` atlas and the `#anim` tag table:

```json
{ "type": "Animation", "layers": ["main"], "anchor_x": 0.5, "anchor_y": 0.5 }
```

Timeline tags become animation tags. This kit uses `idle`, `run`, `attack`,
`skill`, `skill2`, `ult`, `dead` — `action_name()` in `actions.rs` is what
picks `attack`/`skill`/`skill2`/`ult`, so those four must exist by those
names. (Tags may be prefixed, e.g. `wan_idle`, only if a `.data_champion`
sets a matching `anim_prefix`; from pure Rust, use the bare names.)

Without Aseprite, ship the pair by hand instead:

```
champion/avatar_wan#sheet.png         PNG atlas
champion/avatar_wan#anim.fanim        frame rects in pixel coordinates
```

### 2. Skill icons — required for readable UI

```
icons/avatar_wan_skills#sheet.png            the atlas
icons/avatar_wan_skills#data.sprite_sheet    UV lookup table
```

Four tags, in the engine's `<champion_id>_<index>` convention — `avatar_wan_0`
(attack), `_1` (The Avatar Cycle), `_2` (Spirit Step), `_3` (Harmonic Convergence).
The `.sprite_sheet` holds **normalized** 0–1 UVs, like `riot_items_tfm2`'s
item sheet:

```json
{ "images": { "avatar_wan_0": { "x": 0.0, "y": 0.0, "w": 0.25, "h": 0.25 } } }
```

`SKILL_ICON_SHEET` in `constants.rs` is what points at this path — delete the
`skill_icon` override in `champion.rs` to fall back to the base game's sheet.

### 3. Ban/pick illustration — optional

```
banpick_illustrations/avatar_wan.png    512 × 640 sRGB (1024 max per side)
```

Center-cropped to fit; a missing file falls back to the pixel sprite. Note the
docs describe these as *presentation-only packs*, registered with
`"mod_type": "banpick_illustration"` in `mod.mod_info` — worth confirming
whether that can sit on a champion mod or wants its own folder.

### 4. Mod listing images — optional

`thumbnail.png` and `preview.png` in the mod root, as `riot_items_tfm2` has.

### 5. Not reachable from Rust

Per-effect visuals (`view_effects`, `view_projectiles`, `view_buffs`) are
`.data_champion` declarations with no stable-API equivalent, so the element
procs and the burn currently land without their own VFX. Nothing shows which
element Wan is attuned to either, which matters more here than usual since the
whole kit reads off it — a client extension (`StableExtension`) drawing an
indicator is the stable-API route.

## Notes on the simulation

Every gameplay callback runs inside the deterministic sim: same inputs must
give the same result on every machine. Derive randomness only from the
`rng_seed` you are handed — never from clocks, addresses, or global state.

Distances are world units (960,000 × 960,000 map; champion radius ≈ 10,000;
melee range ≈ 12,000) and durations are ticks (60 = 1 second).

Numbers in `constants.rs` come from the kit spec; the ones it did not specify
(attack range, cooldowns, dash distance, base stats) are calibrated against
the SDK's documented example champion, not measured against the live roster —
expect to tune them.

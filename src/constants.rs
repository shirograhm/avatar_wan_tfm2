use crate::element::Element;

// ------------------------------------------------ Defaults
pub const MOD_ID: &str = "avatar_wan_tfm2";
pub const TICKS_PER_SECOND: f64 = 60.0;
macro_rules! ticks {
    ($num:expr) => {
        $num * TICKS_PER_SECOND
    };
}

// ------------------------------------------------ Soul of Raava
pub const ATTACK_MAGIC_SHARE: usize = 50;
pub const ATTACK_PHYSICAL_SHARE: usize = 50;
pub const ATTACK_COOLTIME: usize = 60;

// ------------------------------------------------ Elements
pub const STARTING_ELEMENT: Element = Element::Fire;

pub const AIR_STACK_BUFF: &str = "wan_air_stack";
pub const AIR_ATTACK_SPEED_PERCENT: i32 = 5;
pub const AIR_DURATION: usize = ticks!(2.0) as usize;
pub const AIR_MAX_STACKS: usize = 4;

pub const WATER_MISSING_HP_PERCENT: usize = 5;
pub const WATER_SPLASH_VFX_BUFF: &str = "wan_water_splash";
pub const WATER_SPLASH_VFX_TICKS: usize = 18;

pub const EARTH_ATTACK_SHARE: usize = 50;
pub const EARTH_RADIUS: u64 = 40_000;
pub const EARTH_SPLASH_VFX_BUFF: &str = "wan_earth_splash";
pub const EARTH_SPLASH_VFX_TICKS: usize = 36;

pub const BURN_DAMAGE: usize = 10;
pub const BURN_AD_RATIO: usize = 15;
pub const BURN_AP_RATIO: usize = 35;
pub const BURN_TICKS: usize = 6;
pub const BURN_TICK_INTERVAL: usize = 30;
pub const BURN_VFX_BUFF: &str = "wan_fire_burn";
pub const BURN_VFX_TICKS: usize = BURN_TICKS * BURN_TICK_INTERVAL;

// ------------------------------------------------ Spirit Step
pub const STEP_BUFF: &str = "wan_spirit_step";
pub const STEP_MOVE_SPEED_PERCENT: i32 = 20;
pub const STEP_BUFF_DURATION: usize = ticks!(4.0) as usize;
pub const STEP_STORE_PERCENT: usize = 80;
pub const STEP_HEAL_PERCENT: usize = 80;
pub const STEP_HEAL_FLAT: usize = 35;
pub const STEP_STORE_PREFIX: &str = "wan_step_store";
pub const STEP_DASH_DISTANCE: u64 = 60_000;
pub const STEP_DASH_TICKS: usize = 20;

// ------------------------------------------------ Harmonic Convergence
pub const CONVERGENCE_BUFF: &str = "wan_harmonic_convergence";
pub const CONVERGENCE_DURATION: usize = ticks!(8.0) as usize;
pub const CONVERGENCE_SHIELD: usize = 400;
pub const CONVERGENCE_SHIELD_AP_RATIO: usize = 80;
pub const CONVERGENCE_SHIELD_HP_RATIO: usize = 8;
pub const CONVERGENCE_BASE_SCALE: usize = 100;
// How hurt to assume he is when valuing Water's heal — used by the basic
// attack and by the ult, since neither can read current HP.
pub const EXPECTED_MISSING_HP_PERCENT: usize = 25;
pub const CONVERGENCE_ATTACK_SPEED_PERCENT: usize =
    100 + AIR_MAX_STACKS * AIR_ATTACK_SPEED_PERCENT as usize;

pub fn percent_of(value: usize, percent: usize) -> usize {
    (value * percent) / 100
}

//! Conservative preallocation profile plus exact incremental wire-byte charging.
use super::input::BurrowConfigInput;
use crate::{
    browser_experiments::{
        error, FieldError, Input, MAX_CHECKPOINTS, MAX_EPISODE_BYTES, MAX_INPUT_BYTES,
    },
    burrow,
};
use serde::Serialize;

pub const MAX_CELLS: u64 = 4096;
pub const MAX_AGENTS: u64 = 16;
pub const MAX_OPPORTUNITIES: u64 = 8192;
// Input is retained once. Reserve additional enclosing keys, source/target identity,
// capture metadata and punctuation; component charge includes its delimiter.
const WRAPPER_BYTES: usize = MAX_INPUT_BYTES + 8192;

type Checked<T> = Result<T, Vec<FieldError>>;
fn add(a: u64, b: u64) -> Checked<u64> {
    a.checked_add(b)
        .ok_or_else(|| error("episode", "retention sum overflow"))
}
fn mul(a: u64, b: u64) -> Checked<u64> {
    a.checked_mul(b)
        .ok_or_else(|| error("episode", "retention product overflow"))
}
fn count(value: usize) -> Checked<u64> {
    u64::try_from(value).map_err(|_| error("episode", "count exceeds u64"))
}
struct Shape {
    cells: u64,
    agents: u64,
    resources: u64,
    spoil: u64,
    burrow: bool,
}
fn burrow_shape(config: &BurrowConfigInput, ticks: u32) -> Checked<Shape> {
    let native = config.to_core()?;
    let (width, height, agents, spoil) = match native.fixture {
        burrow::Fixture::Growing {
            width,
            height,
            workers,
        } => (width, height, workers, 0),
        burrow::Fixture::Choice { pile, .. } => {
            native
                .freshness_window
                .checked_mul(2)
                .and_then(|start| start.checked_add(u64::from(ticks)))
                .ok_or_else(|| error("ticks", "requested clock advance overflow"))?;
            (
                9,
                7,
                1,
                if pile == burrow::Pile::SingleFresh {
                    1
                } else {
                    4
                },
            )
        }
        burrow::Fixture::Corridor { length, workers } => {
            // Public fixture helper is safe only after the cheap profile dimension guard.
            if mul(u64::from(length) + 2, 3)? > MAX_CELLS {
                return Err(error("cells", "browser spatial limit is 4096 grid cells"));
            }
            let setup = burrow::fixtures::corridor_setup(length, workers)?;
            (setup.width, setup.height, workers, 0)
        }
    };
    Ok(Shape {
        cells: mul(u64::from(width), u64::from(height))?,
        agents: u64::from(agents),
        resources: 0,
        spoil,
        burrow: true,
    })
}

/// Validate original setup/config before World, trace, or dense-map allocation.
/// The profile is intentionally conservative; it never changes requested inputs.
pub fn preflight(input: &Input) -> Checked<()> {
    let (shape, ticks, sample_every) = match input {
        Input::BurrowExcavation {
            config,
            ticks,
            sample_every,
            ..
        } => (burrow_shape(config, *ticks)?, *ticks, *sample_every),
        Input::BurrowAccess {
            config,
            ticks,
            sample_every,
            ..
        } => {
            config.to_core()?;
            (burrow_shape(&config.lab, *ticks)?, *ticks, *sample_every)
        }
        Input::ForagingFixed {
            setup,
            ticks,
            sample_every,
            ..
        } => {
            setup.to_core()?;
            (
                Shape {
                    cells: mul(u64::from(setup.width), u64::from(setup.height))?,
                    agents: u64::from(setup.agents),
                    resources: count(setup.resources.len())?,
                    spoil: 0,
                    burrow: false,
                },
                *ticks,
                *sample_every,
            )
        }
        Input::ForagingPassage {
            setup,
            ticks,
            sample_every,
            ..
        } => {
            setup.to_core()?;
            (
                Shape {
                    cells: mul(u64::from(setup.width), u64::from(setup.height))?,
                    agents: count(setup.workers.len())?,
                    resources: count(setup.resources.len())?,
                    spoil: 0,
                    burrow: false,
                },
                *ticks,
                *sample_every,
            )
        }
        Input::ForagingConstruction {
            setup,
            ticks,
            sample_every,
            ..
        } => {
            setup.to_core()?;
            (
                Shape {
                    cells: mul(u64::from(setup.width), u64::from(setup.height))?,
                    agents: count(setup.workers.len())?,
                    resources: count(setup.food.len())?,
                    spoil: count(setup.diggable.len())?,
                    burrow: false,
                },
                *ticks,
                *sample_every,
            )
        }
        _ => return Err(error("study", "spatial preflight requires a spatial study")),
    };
    if shape.cells > MAX_CELLS {
        return Err(error("cells", "browser spatial limit is 4096 grid cells"));
    }
    if shape.agents > MAX_AGENTS {
        return Err(error("agents", "browser spatial limit is 16 Agents"));
    }
    if !shape.burrow && !(1..=7200).contains(&ticks) {
        return Err(error("ticks", "must be in 1..=7200"));
    }
    let opportunities = shape
        .agents
        .checked_mul(u64::from(ticks))
        .ok_or_else(|| error("opportunities", "checked product overflow"))?;
    if opportunities > MAX_OPPORTUNITIES {
        return Err(error("opportunities", "browser spatial limit is 8192"));
    }
    if sample_every == 0 {
        return Err(error("sample_every", "must be positive"));
    }
    // Burrow can retain same-tick initial/terminal stages. CPFA rounds up a
    // noncadence terminal boundary and retains its initial boundary exactly once.
    let frames = if ticks == 0 {
        1
    } else if shape.burrow {
        add(2, u64::from(ticks / sample_every))?
    } else {
        add(1, u64::from(ticks).div_ceil(u64::from(sample_every)))?
    };
    if frames > MAX_CHECKPOINTS as u64 {
        return Err(error("checkpoints", "episode exceeds 4096 checkpoints"));
    }

    let estimate = if shape.burrow {
        // A native opportunity contributes bounded event/choice/dig-distance/
        // delivery/carrier records (2048 bytes total). Each checkpoint can repeat
        // its complete own-event/choice prefixes (768 bytes/opportunity across
        // ALL Agents, not per Agent). Cell allowance includes escaped ASCII,
        // decoded researcher geometry and the access route. Original seeded
        // choice spoil and all enclosing snapshot/worker fields are included.
        let native = add(
            mul(add(opportunities, shape.spoil)?, 2048)?,
            mul(shape.cells, 128)?,
        )?;
        let frame = add(
            add(mul(shape.cells, 128)?, mul(opportunities, 768)?)?,
            add(mul(shape.agents, 2048)?, 8192)?,
        )?;
        add(native, mul(frames, frame)?)?
    } else {
        // Charge repeated native AND researcher snapshots, local Agent views,
        // supplied geometry and every Agent's full sparse map. Counts retain
        // complete inventories/access milestones and all resource/waypoint/spoil
        // records, rather than only snapshot_bytes or newly changed cells.
        // Consumable food bounds publications; spoil is bounded by the mask.
        let geometry = mul(shape.cells, add(256, mul(shape.agents, 128)?)?)?;
        let inventory = add(
            mul(shape.resources, 8192)?,
            mul(shape.spoil.min(opportunities), 1024)?,
        )?;
        let frame = add(
            add(geometry, inventory)?,
            add(mul(shape.agents, 16384)?, 16384)?,
        )?;
        // One extra full frame covers native final summary and its per-Agent /
        // access-history prefixes outside the snapshots; setup fits raw limit.
        mul(add(frames, 1)?, frame)?
    };
    if add(estimate, WRAPPER_BYTES as u64)? > MAX_EPISODE_BYTES as u64 {
        return Err(error("episode","conservative complete spatial retention exceeds 16 MiB; reduce horizon, geometry, population or sample frequency"));
    }
    Ok(())
}

/// Shared allowance for all retained wire components, charged BEFORE append.
/// Charge lossless wire values, including repeated native payload and checkpoint
/// data, once per retained occurrence. Never substitute native snapshot_bytes.
/// Final record::check_bounds remains the complete-envelope authority.
pub struct CaptureBudget {
    used: usize,
}
impl Default for CaptureBudget {
    fn default() -> Self {
        Self::new()
    }
}
impl CaptureBudget {
    pub fn new() -> Self {
        Self {
            used: WRAPPER_BYTES,
        }
    }
    pub fn used_bytes(&self) -> usize {
        self.used
    }
    pub fn charge<T: Serialize>(&mut self, value: &T) -> Checked<()> {
        let available = MAX_EPISODE_BYTES
            .checked_sub(self.used)
            .and_then(|left| left.checked_sub(2))
            .ok_or_else(|| error("episode", "complete spatial retention exceeds 16 MiB"))?;
        let bytes =
            crate::browser_experiments::record::serialized_size(value, available, "episode")?;
        self.used = self
            .used
            .checked_add(bytes)
            .and_then(|used| used.checked_add(2))
            .ok_or_else(|| error("episode", "capture byte count overflow"))?;
        Ok(())
    }
}

mod ss;
mod sscollector;
mod ssmutator;
pub mod ssconstraints;
mod sstracelocal;
mod validate;
mod rtm;

const VERBOSE: bool = false;//cfg!(debug_assertions);

const CONCURRENT_TRIGGER: f32 = 0.5;

/** See: The pauseless GC algorithm (Click, C., Tene, G., & Wolf, M.). */
const SELF_HEALING_BARRIER: bool = true;

const RTM_SUPPORTED_EVACUATION: bool = false;
/** Retry the transaction for a few times, then fallback to atomic evacuation */
const RTM_EVACUATION_MAX_RETRY: usize = 3;

pub use self::ss::SemiSpace;
pub use self::ss::PLAN;
pub use self::ssmutator::SSMutator;
pub use self::sstracelocal::SSTraceLocal;
pub use self::sscollector::SSCollector;

pub use self::ss::SelectedPlan;
pub use self::ssconstraints as SelectedConstraints;
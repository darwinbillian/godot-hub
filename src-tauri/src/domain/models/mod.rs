mod flavor;
mod release;
mod variant;
mod version;

pub use flavor::{Flavor, FlavorKind, FlavorKindFlags};
pub use release::{Release, ReleaseVariant};
pub use variant::{Variant, VariantFlags};
pub use version::Version;

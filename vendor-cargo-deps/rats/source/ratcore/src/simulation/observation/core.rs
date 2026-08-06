type SimulationIdentityKind = super::nomenclature::SimulationIdentityKind;
type SimulationIdentityRegistry = super::nomenclature::SimulationIdentityRegistry;
type SimulationProbeId = super::nomenclature::SimulationProbeId;
type SimulationScenarioId = super::nomenclature::SimulationScenarioId;

const PROBE_SCHEMA_VERSION: u16 = 1;
const MAX_METADATA_ENTRIES: usize = 12;
const MAX_METADATA_KEY_BYTES: usize = 48;
const MAX_METADATA_VALUE_BYTES: usize = 96;
const MAX_RECORDS: u32 = 256;
const MAX_DIAGNOSTICS: usize = 64;

include!("model.rs");
include!("observer.rs");
include!("validation.rs");

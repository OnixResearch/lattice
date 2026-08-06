type SimulationChoicePointId = super::nomenclature::SimulationChoicePointId;
type SimulationEffectClassId = super::nomenclature::SimulationEffectClassId;
type SimulationIdentityKind = super::nomenclature::SimulationIdentityKind;
type SimulationIdentityRegistry = super::nomenclature::SimulationIdentityRegistry;

const MAX_CONTROL_INPUTS: usize = 128;
const MAX_CONTROL_INPUTS_U32: u32 = 128;
const MAX_READY_SOURCES: usize = 16;
const MAX_CONTROL_ISSUES: usize = 64;

include!("model.rs");
include!("validation.rs");
include!("provider.rs");
include!("effect.rs");

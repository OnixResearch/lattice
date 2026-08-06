struct PilotProbeEmitter<'a> {
    observer: Option<&'a mut dyn super::observation::SimulationProbeObserver>,
    sequence: u64,
    diagnostics: Vec<SimulationProbeDiagnostic>,
    dropped: u32,
}

impl<'a> PilotProbeEmitter<'a> {
    const fn new(
        observer: Option<&'a mut dyn super::observation::SimulationProbeObserver>,
    ) -> Self {
        Self {
            observer,
            sequence: 0,
            diagnostics: Vec::new(),
            dropped: 0,
        }
    }

    fn emit(&mut self, registry: &SimulationIdentityRegistry, mut record: SimulationProbeRecord) {
        self.sequence = self.sequence.saturating_add(1);
        record.sequence = self.sequence;
        let previous = self.sequence.checked_sub(1).filter(|value| *value > 0);
        let issues = super::observation::validate_probe_record(registry, previous, &record);
        if issues.is_empty() {
            if let Some(observer) = self.observer.as_deref_mut() {
                observer.observe(&record);
            }
        } else {
            self.dropped = self.dropped.saturating_add(1);
            self.diagnostics.extend(issues);
        }
    }
}

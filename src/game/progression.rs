#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunProgress {
    pub unsecured_resource: u32,
    pub banked_resource: u32,
    pub active_anchor_id: Option<String>,
    pub respawn_position: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorActivation {
    pub newly_activated: bool,
    pub banked_amount: u32,
}

impl RunProgress {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_resource(&mut self, amount: u32) -> bool {
        if amount == 0 {
            return false;
        }

        self.unsecured_resource = self.unsecured_resource.saturating_add(amount);
        true
    }

    pub fn activate_anchor(&mut self, anchor_id: &str, position: [f32; 3]) -> AnchorActivation {
        let newly_activated = self.active_anchor_id.as_deref() != Some(anchor_id);
        let banked_amount = self.unsecured_resource;

        self.banked_resource = self.banked_resource.saturating_add(banked_amount);
        self.unsecured_resource = 0;
        self.active_anchor_id = Some(anchor_id.to_string());
        self.respawn_position = Some(position);

        AnchorActivation {
            newly_activated,
            banked_amount,
        }
    }

    pub fn lose_unsecured_on_death(&mut self) -> u32 {
        let lost = self.unsecured_resource;
        self.unsecured_resource = 0;
        lost
    }

    pub fn clear_anchor(&mut self) {
        self.active_anchor_id = None;
        self.respawn_position = None;
    }

    pub fn respawn_position_or(&self, fallback: [f32; 3]) -> [f32; 3] {
        self.respawn_position.unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collecting_resource_adds_to_unsecured_total() {
        let mut progress = RunProgress::new();

        assert!(progress.collect_resource(15));
        assert!(progress.collect_resource(10));

        assert_eq!(progress.unsecured_resource, 25);
        assert_eq!(progress.banked_resource, 0);
    }

    #[test]
    fn anchor_banks_unsecured_resource_and_sets_respawn() {
        let mut progress = RunProgress::new();
        progress.collect_resource(25);

        let activation = progress.activate_anchor("foundation_anchor", [1.0, 2.0, 3.0]);

        assert!(activation.newly_activated);
        assert_eq!(activation.banked_amount, 25);
        assert_eq!(progress.unsecured_resource, 0);
        assert_eq!(progress.banked_resource, 25);
        assert_eq!(
            progress.active_anchor_id.as_deref(),
            Some("foundation_anchor")
        );
        assert_eq!(
            progress.respawn_position_or([0.0, 0.0, 0.0]),
            [1.0, 2.0, 3.0]
        );
    }

    #[test]
    fn death_loses_only_unsecured_resource() {
        let mut progress = RunProgress::new();
        progress.collect_resource(25);
        progress.activate_anchor("anchor", [0.0, 0.0, 0.0]);
        progress.collect_resource(10);

        assert_eq!(progress.lose_unsecured_on_death(), 10);
        assert_eq!(progress.unsecured_resource, 0);
        assert_eq!(progress.banked_resource, 25);
    }
}

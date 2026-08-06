#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_X: u16 = 2;
    const FRAME_Y: u16 = 3;
    const FRAME_WIDTH: u16 = 40;
    const FRAME_HEIGHT: u16 = 12;
    const TINY_WIDTH: u16 = 1;
    const TINY_HEIGHT: u16 = 1;

    fn frame() -> ShellRect {
        ShellRect::new(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT)
    }

    fn close_control() -> ShellControl {
        ShellControl::new("close", ShellControlKind::Close, "panel.close")
    }

    #[test]
    fn validates_positive_shell_model_and_geometry() {
        let panel = PanelShell::new("editor", "Editor", frame())
            .with_active(true)
            .with_controls(vec![close_control()]);
        let model =
            WindowPanelShellModel::new(vec![panel], Some(crate::docking::PanelId::new("editor")));

        let projection = validate_window_panel_shell_model(&model);

        assert!(projection.diagnostics.is_empty());
        assert_eq!(projection.panels[0].title, "Editor");
        assert_eq!(
            projection.panels[0].geometry.header.height,
            DEFAULT_HEADER_HEIGHT
        );
        assert!(!projection.panels[0].geometry.client.is_empty());
    }

    #[test]
    fn invalid_shell_state_reports_diagnostics_and_safe_fallbacks() {
        let invalid = PanelShell::new("editor", "", frame())
            .with_active(true)
            .with_visible(false)
            .with_controls(vec![ShellControl::without_action(
                "broken",
                ShellControlKind::Custom("broken".to_string()),
            )]);
        let duplicate = PanelShell::new("editor", "Duplicate", frame());
        let model = WindowPanelShellModel::new(
            vec![invalid, duplicate],
            Some(crate::docking::PanelId::new("missing")),
        );

        let projection = validate_window_panel_shell_model(&model);

        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::DuplicatePanelId })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::StaleFocus })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::EmptyTitle })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::HiddenActivePanel })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::InvalidControlAction })
        );
        assert_eq!(projection.panels[0].title, UNTITLED_PANEL);
        assert!(!projection.panels[0].active);
        assert!(!projection.panels[0].controls[0].enabled);
    }

    #[test]
    fn tiny_and_control_overflow_geometry_fail_closed() {
        let tiny = ShellRect::new(FRAME_X, FRAME_Y, TINY_WIDTH, TINY_HEIGHT);
        let controls = vec![close_control(), close_control(), close_control()];

        let panel_id = crate::docking::PanelId::new("tiny");
        let geometry = compute_window_panel_shell_geometry(
            tiny,
            &controls,
            ShellFrameOptions::default(),
            Some(&panel_id),
        );

        assert!(geometry.client.is_empty());
        assert!(
            geometry
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::TinyFrame })
        );
        assert!(
            geometry
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::ControlOverflow })
        );
    }

    #[test]
    fn renderer_projections_share_shell_semantics() {
        let panel =
            PanelShell::new("editor", "Editor", frame()).with_controls(vec![close_control()]);
        let model = WindowPanelShellModel::new(vec![panel], None);

        let web = project_window_panel_shell_renderer(ShellRenderer::Meteorite, &model);
        let tui = project_window_panel_shell_renderer(ShellRenderer::Subwayrat, &model);

        assert_eq!(web.projection, tui.projection);
    }
}

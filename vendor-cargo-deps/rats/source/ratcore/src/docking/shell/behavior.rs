/// Validates a shell model and computes safe renderable geometry.
#[must_use]
pub fn validate_window_panel_shell_model(
    model: &WindowPanelShellModel,
) -> WindowPanelShellProjection {
    let diagnostic_reserve = model.panels.len().saturating_mul(4).saturating_add(1);
    let mut diagnostics = Vec::with_capacity(diagnostic_reserve);
    let duplicate_ids = duplicate_panel_ids(&model.panels);
    for panel_id in &duplicate_ids {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::DuplicatePanelId,
            Some(panel_id.clone()),
            None,
            "duplicate panel id",
        ));
    }
    if let Some(focused_panel) = &model.focused_panel
        && !model
            .panels
            .iter()
            .any(|panel| panel.panel_id == *focused_panel)
    {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::StaleFocus,
            Some(focused_panel.clone()),
            None,
            "focused panel is absent",
        ));
    }

    let mut panels = Vec::with_capacity(model.panels.len());
    debug_assert!(panels.capacity() >= model.panels.len());
    for panel in &model.panels {
        let (projection, mut panel_diagnostics) = project_panel_shell(panel);
        diagnostics.append(&mut panel_diagnostics);
        panels.push(projection);
    }
    debug_assert_eq!(panels.len(), model.panels.len());

    WindowPanelShellProjection {
        panels,
        diagnostics,
    }
}

fn project_panel_shell(panel: &PanelShell) -> (PanelShellProjection, Vec<ShellDiagnostic>) {
    let (title, title_diagnostic) = safe_title(panel);
    let (is_active, active_diagnostic) = safe_active(panel);
    let (controls, mut diagnostics) = safe_controls(panel);
    debug_assert_eq!(controls.len(), panel.controls.len());
    diagnostics.extend(title_diagnostic);
    diagnostics.extend(active_diagnostic);
    let mut geometry = compute_window_panel_shell_geometry(
        panel.frame,
        &controls,
        ShellFrameOptions::default(),
        Some(&panel.panel_id),
    );
    diagnostics.append(&mut geometry.diagnostics);
    debug_assert!(diagnostics.len() <= controls.len().saturating_add(4));
    (
        PanelShellProjection {
            panel_id: panel.panel_id.clone(),
            title,
            active: is_active,
            z_order: panel.z_order,
            visible: panel.visible,
            controls,
            geometry,
            client_slot_id: panel.client_slot_id.clone(),
        },
        diagnostics,
    )
}

/// Computes deterministic frame geometry for a shell.
#[must_use]
pub fn compute_window_panel_shell_geometry(
    frame: ShellRect,
    controls: &[ShellControl],
    options: ShellFrameOptions,
    panel_id: Option<&ShellPanelId>,
) -> ShellGeometry {
    let header_height = options.header_height.min(frame.height);
    let header = ShellRect::new(frame.x, frame.y, frame.width, header_height);
    let body_y = frame.y.saturating_add(header_height);
    let body_height = frame.height.saturating_sub(header_height);
    let body = ShellRect::new(frame.x, body_y, frame.width, body_height);
    let edge_total = options.border_thickness.saturating_mul(2);
    let client_x = frame.x.saturating_add(options.border_thickness);
    let client_y = body.y.saturating_add(options.border_thickness);
    let client_width = body.width.saturating_sub(edge_total);
    let client_height = body.height.saturating_sub(options.border_thickness);
    let client = ShellRect::new(client_x, client_y, client_width, client_height);
    let mut diagnostics = Vec::with_capacity(controls.len().saturating_add(1));
    if frame.is_empty() || client.is_empty() {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::TinyFrame,
            panel_id.cloned(),
            None,
            "frame is too small for preferred shell geometry",
        ));
    }
    let (control_regions, mut control_diagnostics) =
        compute_control_regions(frame, controls, options, panel_id);
    diagnostics.append(&mut control_diagnostics);
    debug_assert!(header.height <= frame.height);
    debug_assert!(body.height <= frame.height);
    let edge_regions = compute_resize_regions(frame, options.resize_affordance);
    ShellGeometry {
        frame,
        header,
        body,
        client,
        controls: control_regions,
        resize_regions: edge_regions,
        diagnostics,
    }
}

/// Wraps the shared shell projection for a renderer.
#[must_use]
pub fn project_window_panel_shell_renderer(
    renderer: ShellRenderer,
    model: &WindowPanelShellModel,
) -> ShellRendererProjection {
    ShellRendererProjection {
        renderer,
        projection: validate_window_panel_shell_model(model),
    }
}

fn duplicate_panel_ids(panels: &[PanelShell]) -> std::collections::BTreeSet<ShellPanelId> {
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = std::collections::BTreeSet::new();
    for panel in panels {
        if !seen.insert(panel.panel_id.clone()) {
            duplicates.insert(panel.panel_id.clone());
        }
    }
    duplicates
}

fn safe_title(panel: &PanelShell) -> (String, Option<ShellDiagnostic>) {
    if panel.title.trim().is_empty() {
        return (
            UNTITLED_PANEL.to_string(),
            Some(ShellDiagnostic::new(
                ShellDiagnosticCode::EmptyTitle,
                Some(panel.panel_id.clone()),
                None,
                "panel title is empty",
            )),
        );
    }
    (panel.title.clone(), None)
}

fn safe_active(panel: &PanelShell) -> (bool, Option<ShellDiagnostic>) {
    if panel.active && !panel.visible {
        return (
            false,
            Some(ShellDiagnostic::new(
                ShellDiagnosticCode::HiddenActivePanel,
                Some(panel.panel_id.clone()),
                None,
                "hidden panel cannot be active",
            )),
        );
    }
    (panel.active, None)
}

fn safe_controls(panel: &PanelShell) -> (Vec<ShellControl>, Vec<ShellDiagnostic>) {
    let mut diagnostics = Vec::with_capacity(panel.controls.len());
    debug_assert!(diagnostics.capacity() >= panel.controls.len());
    let controls = panel
        .controls
        .iter()
        .map(|control| {
            if control.action_id.is_none() {
                diagnostics.push(ShellDiagnostic::new(
                    ShellDiagnosticCode::InvalidControlAction,
                    Some(panel.panel_id.clone()),
                    Some(control.id.clone()),
                    "control has no action target",
                ));
                return control.clone().with_enabled(false);
            }
            control.clone()
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(controls.len(), panel.controls.len());
    (controls, diagnostics)
}

fn compute_control_regions(
    frame: ShellRect,
    controls: &[ShellControl],
    options: ShellFrameOptions,
    panel_id: Option<&ShellPanelId>,
) -> (Vec<ShellControlRegion>, Vec<ShellDiagnostic>) {
    let mut regions = Vec::with_capacity(controls.len());
    let mut diagnostics = Vec::with_capacity(controls.len());
    debug_assert!(regions.capacity() >= controls.len());
    let mut right_edge = frame.x.saturating_add(frame.width);
    for control in controls {
        let step = options.control_width.saturating_add(options.control_gap);
        right_edge = right_edge.saturating_sub(step);
        let rect = ShellRect::new(
            right_edge,
            frame.y,
            options.control_width.min(frame.width),
            options.header_height.min(frame.height),
        );
        let is_visible = rect.x >= frame.x && !rect.is_empty();
        if !is_visible {
            diagnostics.push(ShellDiagnostic::new(
                ShellDiagnosticCode::ControlOverflow,
                panel_id.cloned(),
                Some(control.id.clone()),
                "control does not fit in header",
            ));
        }
        regions.push(ShellControlRegion {
            control_id: control.id.clone(),
            rect,
            visible: is_visible,
        });
    }
    debug_assert_eq!(regions.len(), controls.len());
    (regions, diagnostics)
}

fn compute_resize_regions(frame: ShellRect, thickness: u16) -> ShellResizeRegions {
    let right_x = frame
        .x
        .saturating_add(frame.width.saturating_sub(thickness));
    let bottom_y = frame
        .y
        .saturating_add(frame.height.saturating_sub(thickness));
    ShellResizeRegions {
        top: ShellRect::new(frame.x, frame.y, frame.width, thickness.min(frame.height)),
        bottom: ShellRect::new(frame.x, bottom_y, frame.width, thickness.min(frame.height)),
        left: ShellRect::new(frame.x, frame.y, thickness.min(frame.width), frame.height),
        right: ShellRect::new(right_x, frame.y, thickness.min(frame.width), frame.height),
    }
}

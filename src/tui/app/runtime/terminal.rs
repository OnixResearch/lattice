//! Terminal event loop and action dispatch.

use super::*;

impl State {
    /// Run the TUI event loop.
    pub async fn run_tui(mut self) -> io::Result<()> {
        // Terminal setup.
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        crossterm::execute!(stdout, event::EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        info!("TUI started");

        loop {
            self.poll_run_events();

            // Render.
            terminal.draw(|frame| {
                panels::render(&mut self, frame.area(), frame.buffer_mut());
            })?;

            // Poll for events (16ms → ~60fps).
            if event::poll(std::time::Duration::from_millis(16))? {
                let ev = event::read()?;
                if let Some(action) = input::handle_event(&mut self, &ev)
                    && self.apply_action(action)
                {
                    break;
                }
            }
        }

        // Cleanup.
        crossterm::execute!(terminal.backend_mut(), event::DisableMouseCapture)?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::terminal::disable_raw_mode()?;

        info!("TUI exited");
        Ok(())
    }
}

impl State {
    fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => {
                return self.request_quit();
            }
            Action::ConfirmQuit => return true,
            Action::CancelQuit => {
                self.cancel_quit_confirmation();
            }
            Action::RunWorkflow => {
                self.start_workflow_run();
            }
            Action::CancelRun => {
                self.cancel_active_run();
            }
            Action::OpenRunHistory => {
                self.open_run_history();
            }
            Action::OpenApprovals => {
                self.open_approvals();
            }
            Action::OpenDiagnostics => {
                self.open_diagnostics();
            }
            Action::SaveWorkflow => {
                self.save_workflow();
            }
            Action::AddNode => {
                self.open_add_node_flow();
            }
            Action::OpenWorkflow => {
                self.open_workflow_browser();
            }
            Action::OpenWorkflowTemplates => {
                self.open_workflow_template_browser();
            }
            Action::AutoLayout => {
                self.auto_layout();
            }
            Action::EditNodeLabel => {
                self.start_property_edit(PropertyEditField::Label);
            }
            Action::EditNodeCommand => {
                self.start_property_edit(PropertyEditField::Command);
            }
            Action::EditNodeTimeout => {
                self.start_property_edit(PropertyEditField::TimeoutSecs);
            }
            Action::ProbeCredentials => {
                self.probe_selected_node_credentials();
            }
            Action::OpenAuthoring(family) => self.open_authoring(family),
        }
        false
    }
}

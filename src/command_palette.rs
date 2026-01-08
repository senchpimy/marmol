use eframe::egui;

pub enum CommandAction {
    None,
    OpenIconInstaller,
    CreateKanban,
    CreateExcalidraw,
    CreateCanvas,
    CloseTab,
    ToggleLeftMenu,
    Quit,
    CreateFile(String),
}

pub struct Command {
    pub name: String,
    pub action: CommandAction,
}

enum PaletteState {
    CommandSelection,
    Prompt(String, fn(String) -> CommandAction),
}

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub commands: Vec<Command>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    initialized: bool,
    state: PaletteState,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            commands: vec![
                Command {
                    name: "Install Icon Pack".to_string(),
                    action: CommandAction::OpenIconInstaller,
                },
                Command {
                    name: "Create Kanban Board".to_string(),
                    action: CommandAction::CreateKanban,
                },
                Command {
                    name: "Create Excalidraw Drawing".to_string(),
                    action: CommandAction::CreateExcalidraw,
                },
                Command {
                    name: "Create Canvas".to_string(),
                    action: CommandAction::CreateCanvas,
                },
                Command {
                    name: "New File".to_string(),
                    action: CommandAction::None, // Will be handled specially to enter Prompt state
                },
                Command {
                    name: "Close Tab".to_string(),
                    action: CommandAction::CloseTab,
                },
                Command {
                    name: "Toggle Left Menu".to_string(),
                    action: CommandAction::ToggleLeftMenu,
                },
                Command {
                    name: "Quit".to_string(),
                    action: CommandAction::Quit,
                },
            ],
            filtered_indices: vec![],
            selected_index: 0,
            initialized: false,
            state: PaletteState::CommandSelection,
        }
    }
}

impl CommandPalette {
    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
        self.initialized = true;
        self.state = PaletteState::CommandSelection;
        self.update_filter();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.state = PaletteState::CommandSelection;
    }

    fn update_filter(&mut self) {
        if let PaletteState::CommandSelection = self.state {
            let q = self.query.to_lowercase();
            self.filtered_indices = self.commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| q.is_empty() || cmd.name.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect();

            if self.selected_index >= self.filtered_indices.len() {
                self.selected_index = 0;
            }
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) -> CommandAction {
        let mut action_to_perform = CommandAction::None;

        if !self.is_open {
            return action_to_perform;
        }

        let modal = egui::Window::new("Command Palette")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .fixed_size([500.0, 300.0])
            .title_bar(false)
            .collapsible(false)
            .resizable(false);

        modal.show(ctx, |ui| {
            match &mut self.state {
                PaletteState::CommandSelection => {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Command Palette").strong());
                    });
                    ui.add_space(5.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Type a command...")
                            .lock_focus(true)
                    );

                    if self.initialized {
                        response.request_focus();
                        self.initialized = false;
                    }

                    if response.changed() {
                        self.update_filter();
                        self.selected_index = 0;
                    }

                    // Keyboard navigation
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        if !self.filtered_indices.is_empty() && self.selected_index + 1 < self.filtered_indices.len() {
                            self.selected_index += 1;
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.filtered_indices.is_empty() {
                            let real_index = self.filtered_indices[self.selected_index];
                            let cmd_name = self.commands[real_index].name.clone();
                            let cmd_action = match &self.commands[real_index].action {
                                CommandAction::OpenIconInstaller => CommandAction::OpenIconInstaller,
                                CommandAction::CreateKanban => CommandAction::CreateKanban,
                                CommandAction::CreateExcalidraw => CommandAction::CreateExcalidraw,
                                CommandAction::CreateCanvas => CommandAction::CreateCanvas,
                                CommandAction::CloseTab => CommandAction::CloseTab,
                                CommandAction::ToggleLeftMenu => CommandAction::ToggleLeftMenu,
                                CommandAction::Quit => CommandAction::Quit,
                                _ => CommandAction::None,
                            };

                            if cmd_name == "New File" {
                                self.state = PaletteState::Prompt("File name:".to_string(), CommandAction::CreateFile);
                                self.query.clear();
                                self.initialized = true;
                            } else {
                                action_to_perform = cmd_action;
                                self.close();
                            }
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.close();
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                        let mut should_close = false;
                        let mut enter_prompt = None;

                        for (i, &real_index) in self.filtered_indices.iter().enumerate() {
                            let cmd = &self.commands[real_index];
                            let is_selected = i == self.selected_index;
                            
                            let label = ui.selectable_label(is_selected, &cmd.name);
                            if is_selected {
                                label.scroll_to_me(Some(egui::Align::Center));
                            }
                            if label.clicked() {
                                if cmd.name == "New File" {
                                    enter_prompt = Some(PaletteState::Prompt("File name:".to_string(), CommandAction::CreateFile));
                                } else {
                                    action_to_perform = match cmd.action {
                                        CommandAction::OpenIconInstaller => CommandAction::OpenIconInstaller,
                                        CommandAction::CreateKanban => CommandAction::CreateKanban,
                                        CommandAction::CreateExcalidraw => CommandAction::CreateExcalidraw,
                                        CommandAction::CreateCanvas => CommandAction::CreateCanvas,
                                        CommandAction::CloseTab => CommandAction::CloseTab,
                                        CommandAction::ToggleLeftMenu => CommandAction::ToggleLeftMenu,
                                        CommandAction::Quit => CommandAction::Quit,
                                        _ => CommandAction::None,
                                    };
                                    should_close = true;
                                }
                            }
                        }

                        if let Some(new_state) = enter_prompt {
                            self.state = new_state;
                            self.query.clear();
                            self.initialized = true;
                        } else if should_close {
                            self.close();
                        }
                        
                        if self.filtered_indices.is_empty() {
                            ui.weak("No matching commands found.");
                        }
                    });
                }
                PaletteState::Prompt(label, constructor) => {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(label.clone()).strong());
                    });
                    ui.add_space(5.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Enter value...")
                            .lock_focus(true)
                    );

                    if self.initialized {
                        response.request_focus();
                        self.initialized = false;
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        action_to_perform = constructor(self.query.clone());
                        self.close();
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.state = PaletteState::CommandSelection;
                        self.query.clear();
                        self.update_filter();
                        self.initialized = true;
                    }
                }
            }
        });

        ctx.move_to_top(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("Command Palette"),
        ));

        action_to_perform
    }
}

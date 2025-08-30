use crossterm::event::KeyCode;
use mule_macho::{LoadCommand, Macho, Section64};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    InteractiveCommand,
    view::{style_focus, style_normal},
};

#[derive(PartialEq, Copy, Clone)]
enum Focus {
    None,
    Header,
    LoadCommands,
}

static FOCUS_CYCLE_ORDER: [Focus; 2] = [Focus::Header, Focus::LoadCommands];

pub struct MachoInteractiveState {
    previous_focus: Focus,
    focus_on: Focus,
    command_list_state: ListState,
}

impl MachoInteractiveState {
    pub fn new() -> MachoInteractiveState {
        let mut command_list_state = ListState::default();
        command_list_state.select(Some(0));
        MachoInteractiveState {
            command_list_state,
            previous_focus: Focus::None,
            focus_on: Focus::LoadCommands,
        }
    }

    pub fn handle_command(&mut self, command: InteractiveCommand) {
        match command {
            InteractiveCommand::Key(key) => {
                match key {
                    KeyCode::Tab => self.move_focus(1),
                    KeyCode::BackTab => self.move_focus(-1),
                    KeyCode::Down => {
                        if self.focus_on == Focus::LoadCommands {
                            self.command_list_state.select_next();
                        }
                    }
                    KeyCode::Up => {
                        if self.focus_on == Focus::LoadCommands {
                            self.command_list_state.select_previous();
                        }
                    }
                    _ => { /* ignore */ }
                }
            }
            InteractiveCommand::Focus => {
                self.focus_on = self.previous_focus;
            }
            InteractiveCommand::Unfocus => {
                self.previous_focus = self.focus_on;
                self.focus_on = Focus::None;
            }
        }
    }

    fn move_focus(&mut self, dir: isize) {
        let mut ix_focus = 0;
        for i in 0..FOCUS_CYCLE_ORDER.len() {
            if FOCUS_CYCLE_ORDER[i] == self.focus_on {
                ix_focus = i as isize;
            }
        }
        ix_focus += dir;
        let ix = if ix_focus < 0 {
            (FOCUS_CYCLE_ORDER.len() as isize + ix_focus) as usize
        } else {
            ix_focus as usize % FOCUS_CYCLE_ORDER.len()
        };

        self.focus_on = FOCUS_CYCLE_ORDER[ix];
    }
}

pub struct MachoWidget<'a> {
    pub macho: &'a Macho,
    pub state: &'a mut MachoInteractiveState,
}

impl<'a> MachoWidget<'a> {
    pub fn new(macho: &'a Macho, state: &'a mut MachoInteractiveState) -> MachoWidget<'a> {
        MachoWidget { macho, state }
    }

    fn focus_style(&self, focus: Focus) -> Style {
        if self.state.focus_on == focus {
            style_focus()
        } else {
            style_normal()
        }
    }
}

impl<'a> Widget for &mut MachoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_layout =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [content_file, content_detail] = content_layout.areas(area);

        let file_layout = Layout::vertical([Constraint::Max(3), Constraint::Fill(1)]);
        let [mach_header, mach_commands] = file_layout.areas(content_file);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::Header))
            .title("Header");

        Paragraph::new(format!(
            "cpu:{:?} | sub:{:?} | file:{:?}",
            self.macho.header.cpu_type, self.macho.header.cpu_sub_type, self.macho.header.file_type,
        ))
        .block(header_block)
        .render(mach_header, buf);

        let command_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::LoadCommands))
            .title(format!("Load Commands ({})", self.macho.header.no_cmds));

        let cmd_list = List::new(command_list(self.macho))
            .block(command_block)
            .highlight_style(Style::new().black().on_white());
        StatefulWidget::render(
            cmd_list,
            mach_commands,
            buf,
            &mut self.state.command_list_state,
        );

        let detail_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Details");

        let selected = self.state.command_list_state.selected();
        if let Some(selected_pos) = selected {
            let load_command = &self.macho.load_commands[selected_pos];
            if let LoadCommand::Segment64(segment) = load_command {
                let sec_list = List::new(section_list(&segment.sections))
                    .block(detail_block)
                    .highlight_style(Style::new().black().on_white());
                let mut dummy_state = ListState::default();
                StatefulWidget::render(sec_list, content_detail, buf, &mut dummy_state);
            }
        }
    }
}

fn section_list(segs: &[Section64]) -> Vec<&str> {
    let mut result = Vec::with_capacity(segs.len());
    for seg in segs {
        result.push(seg.name.as_str());
    }
    result
}

fn command_list(macho: &Macho) -> Vec<String> {
    let mut result = Vec::with_capacity(macho.load_commands.len());

    for cmd in &macho.load_commands {
        let cmd_str = match cmd {
            LoadCommand::Symtab(_) => "Symtab".to_string(),
            LoadCommand::Dsymtab(_) => "Dsymtab".to_string(),
            LoadCommand::LoadDylib(dylib) => {
                format!("LoadDylib | {}", dylib.name)
            }
            LoadCommand::Dylinker(dylink) => {
                format!("Dylinker | {}", dylink.name)
            }
            LoadCommand::Segment64(seg) => {
                format!("Segment64 | {}", seg.name)
            }
            LoadCommand::Uuid(_) => "UUID".to_string(),
            LoadCommand::CodeSignature(_) => "CodeSignature".to_string(),
            LoadCommand::BuildVersion(_) => "BuildVersion".to_string(),
            LoadCommand::FunctionStarts(_) => "FunctionStarts".to_string(),
            LoadCommand::DataInCode(_) => "DataInCode".to_string(),
            LoadCommand::SourceVersion(_) => "SourceVersion".to_string(),
            LoadCommand::DyldInfoOnly(_) => "DyldInfoOnly".to_string(),
            LoadCommand::Main(_) => "Main".to_string(),
            LoadCommand::LinkeditData(_) => "LinkeditData".to_string(),
            LoadCommand::Unknow(_) => "Unknown".to_string(),
        };
        result.push(cmd_str);
    }

    result
}

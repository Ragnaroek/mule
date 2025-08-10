use macho::{LoadCommand, Macho, Segment64Command, SymtabCommand};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph, StatefulWidget, Widget},
};

pub struct MachoWidget<'a> {
    pub macho: &'a Macho,
    pub command_state: ListState,
}

impl<'a> MachoWidget<'a> {
    pub fn new(macho: &'a Macho) -> MachoWidget<'a> {
        let mut command_state = ListState::default();
        command_state.select(Some(0));

        MachoWidget {
            macho,
            command_state,
        }
    }
}

impl<'a> Widget for &mut MachoWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_layout =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [content_file, content_detail] = content_layout.areas(area);

        // TODO Hights have to be computed dynamically from the Mach-O file
        let file_layout = Layout::vertical([
            Constraint::Max(3),
            Constraint::Max((self.macho.load_commands.len() + 2) as u16),
        ]);
        let [mach_header, mach_commands] = file_layout.areas(content_file);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Header");

        Paragraph::new(format!(
            "cpu:{:?} | sub:{:?} | file:{:?}",
            self.macho.header.cpu_type, self.macho.header.cpu_sub_type, self.macho.header.file_type,
        ))
        .block(header_block)
        .render(mach_header, buf);

        let command_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(format!("Load Commands ({})", self.macho.header.no_cmds));

        let cmd_list = List::new(command_list(self.macho))
            .block(command_block)
            .highlight_style(Style::new().black().on_white());
        StatefulWidget::render(cmd_list, mach_commands, buf, &mut self.command_state);

        Block::bordered()
            .border_type(BorderType::Plain)
            .title("Details")
            .render(content_detail, buf);
    }
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
            LoadCommand::LoadDylinker(_) => "LoadDylinker".to_string(),
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
            LoadCommand::Unknow(_) => "Unknown".to_string(),
        };
        result.push(cmd_str);
    }

    result
}

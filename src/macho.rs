use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, Widget},
};

pub struct MachoWidget {}

impl Widget for MachoWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_layout =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [content_file, content_detail] = content_layout.areas(area);

        // TODO Hights have to be computed dynamically from the Mach-O file
        let file_layout =
            Layout::vertical([Constraint::Max(3), Constraint::Max(10), Constraint::Max(30)]);
        let [mach_header, mach_commands, mach_segments] = file_layout.areas(content_file);

        // TODO put this into the macho.rs file
        Block::bordered()
            .border_type(BorderType::Plain)
            //.style(Style::new().black().on_white())
            .title("Header")
            .render(mach_header, buf);

        Block::bordered()
            .border_type(BorderType::Plain)
            .style(Style::new().black().on_white())
            .title("Load Commands")
            .render(mach_commands, buf);

        Block::bordered()
            .border_type(BorderType::Plain)
            //.style(Style::new().black().on_white())
            .title("Segments")
            .render(mach_segments, buf);

        Block::bordered()
            .border_type(BorderType::Plain)
            .title("Details")
            .render(content_detail, buf);
    }
}

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Paragraph, WidgetRef},
};

pub struct Hex<'a> {
    data: &'a Vec<u8>,
    block: Option<Block<'a>>,
}

impl<'a> Hex<'a> {
    pub fn new(data: &'a Vec<u8>) -> Hex<'a> {
        Hex { data, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn render_hex(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(hex_data_string(self.data, area.width))
            //.scroll((0, 4))
            .render_ref(area, buf);

        //let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        //let mut scrollbar_state = ScrollbarState::new(1000);
        //StatefulWidget::render(scrollbar, area, buf, &mut scrollbar_state);
    }
}

impl<'a> WidgetRef for Hex<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if let Some(block) = &self.block {
            block.render_ref(area, buf);
            let inner = block.inner(area);
            self.render_hex(inner, buf);
        } else {
            self.render_hex(area, buf);
        };
    }
}

fn hex_data_string(data: &Vec<u8>, width: u16) -> String {
    let mut hex_string = String::new();
    let line_info_width = 3 + 3;
    let byte_blocks_per_line = (width - line_info_width) / 9;

    let mut lines = 0;
    let mut offset = 0;
    while offset < data.len() {
        hex_string.push_str(&format!("{:03X}   ", lines));
        for _ in 0..byte_blocks_per_line {
            hex_string.push_str(&format_block(data, offset));
            hex_string.push(' ');
            offset += 4;
        }
        hex_string.push('\n');
        lines += 1;
    }
    hex_string
}

fn format_block(data: &Vec<u8>, offset: usize) -> String {
    let mut block_str = String::new();
    for i in 0..4 {
        if (offset + i) >= data.len() {
            break;
        }
        block_str.push_str(&format!("{:02X}", data[offset + i]));
    }
    block_str
}

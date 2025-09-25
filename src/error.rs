use crate::lexer as lx;

pub struct LineInfo {
    lines: Vec<String>,
    start: usize,
    mark_left: usize,
    mark_right: usize,
    line_no: usize,
}

impl LineInfo {
    pub fn new(
        lines: Vec<String>,
        start: usize,
        mark_left: usize,
        mark_right: usize,
        line_no: usize,
    ) -> LineInfo {
        LineInfo {
            lines,
            start,
            mark_left,
            mark_right,
            line_no,
        }
    }

    pub fn pretty_print(&self) {
        let last_line_no = self.line_no + self.lines.len().saturating_sub(1);
        let ln_width = last_line_no.to_string().len();
        let rel_left = self.mark_left.saturating_sub(self.start);
        let rel_right = self.mark_right.saturating_sub(self.start);

        for (i, line) in self.lines.iter().enumerate() {
            let current_no = self.line_no + i;
            println!("{:>width$} | {}", current_no, line, width = ln_width);

            print!("{:>width$} | ", "", width = ln_width);

            let mut byte_cursor = 0usize;
            let mut mark_chars = 0usize;
            let mut chars_before_mark = 0usize;

            for ch in line.chars() {
                let ch_start = byte_cursor;
                let ch_end = byte_cursor + ch.len_utf8();
                if ch_end <= rel_left {
                    chars_before_mark += 1;
                }
                if rel_right > ch_start && rel_left < ch_end {
                    mark_chars += 1;
                }
                byte_cursor = ch_end;
            }

            if mark_chars == 0 {
                println!();
                continue;
            }

            for _ in 0..chars_before_mark {
                print!(" ");
            }
            for _ in 0..mark_chars.max(1) {
                print!("^");
            }
            println!();
        }
    }
}

pub struct FErrorManager {
    file: String,
}

impl FErrorManager {
    pub fn new(file: String) -> FErrorManager {
        FErrorManager { file }
    }

    pub fn section_at_pos(&self, pos: &lx::Span) -> LineInfo {
        let after_end = match self.file[pos.end..].find('\n') {
            Some(rel) => pos.end + rel + 1,
            None => self.file.len(),
        };

        let before_start = match self.file[..pos.start].rfind('\n') {
            Some(idx) => idx + 1,
            None => 0,
        };

        let line_no = self.file[..before_start]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
            + 1;

        let slice = &self.file[before_start..after_end];
        let mut lines = Vec::new();
        for raw_line in slice.split_inclusive('\n') {
            let text = if raw_line.ends_with('\n') {
                &raw_line[..raw_line.len() - 1]
            } else {
                raw_line
            };
            lines.push(text.to_string());
        }

        LineInfo::new(lines, before_start, pos.start, pos.end, line_no)
    }

    pub fn print_error(&self, pos: &lx::Span, err: String) {
        self.section_at_pos(pos).pretty_print();
        println!("ERROR: {err}");
    }
}

#[macro_export]
macro_rules! debug {
     ($($arg:tt)*)=> {
        #[allow(unused)]
        {
            #[cfg(debug_assertions)]
            {
                println!($($arg)*);
            }
            #[cfg(not(debug_assertions))]
            {
                ( &$($arg)* );
            }
        }
    };
}

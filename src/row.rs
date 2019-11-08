use std::convert::TryInto as _;

#[derive(Clone, Debug)]
pub struct Row {
    cells: Vec<crate::cell::Cell>,
    wrapped: bool,
}

impl Row {
    pub fn new(cols: u16) -> Self {
        Self {
            cells: vec![crate::cell::Cell::default(); cols as usize],
            wrapped: false,
        }
    }

    pub fn clear(&mut self, bgcolor: crate::attrs::Color) {
        for cell in &mut self.cells {
            cell.clear(bgcolor);
        }
        self.wrapped = false;
    }

    fn cells(&self) -> impl Iterator<Item = &crate::cell::Cell> {
        self.cells.iter()
    }

    pub fn cells_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut crate::cell::Cell> {
        self.cells.iter_mut()
    }

    pub fn get(&self, col: u16) -> Option<&crate::cell::Cell> {
        self.cells.get(col as usize)
    }

    pub fn get_mut(&mut self, col: u16) -> Option<&mut crate::cell::Cell> {
        self.cells.get_mut(col as usize)
    }

    pub fn insert(&mut self, i: usize, cell: crate::cell::Cell) {
        self.cells.insert(i, cell);
    }

    pub fn remove(&mut self, i: usize) {
        self.cells.remove(i);
    }

    pub fn truncate(&mut self, len: usize) {
        self.cells.truncate(len);
    }

    pub fn resize(&mut self, len: usize, cell: crate::cell::Cell) {
        self.cells.resize(len, cell);
    }

    pub fn wrap(&mut self, wrap: bool) {
        self.wrapped = wrap;
    }

    pub fn wrapped(&self) -> bool {
        self.wrapped
    }

    pub fn contents(&self, start: u16, width: u16) -> String {
        let mut prev_was_wide = false;
        let mut contents = String::new();

        for cell in self
            .cells()
            .skip(start as usize)
            .take(width.min(self.content_width(start)) as usize)
        {
            if prev_was_wide {
                prev_was_wide = false;
                continue;
            }

            contents += if cell.has_contents() {
                cell.contents()
            } else {
                " "
            };
            prev_was_wide = cell.is_wide();
        }

        contents.trim_end().to_string()
    }

    pub fn contents_formatted(
        &self,
        start: u16,
        width: u16,
        attrs: crate::attrs::Attrs,
    ) -> (Vec<u8>, crate::attrs::Attrs, u16) {
        let mut prev_was_wide = false;
        let mut contents = vec![];
        let mut prev_attrs = attrs;

        let mut cols = 0;
        for cell in self
            .cells()
            .skip(start as usize)
            .take(width.min(self.content_width(start)) as usize)
        {
            if prev_was_wide {
                prev_was_wide = false;
                continue;
            }

            let attrs = cell.attrs();
            if &prev_attrs != attrs {
                contents.append(&mut attrs.escape_code_diff(&prev_attrs));
                prev_attrs = *attrs;
            }

            contents.extend(if cell.has_contents() {
                cell.contents().as_bytes()
            } else if cell.bgcolor() == crate::attrs::Color::Default {
                &b"\x1b[C"[..]
            } else {
                &b"\x1b[X\x1b[C"[..]
            });

            prev_was_wide = cell.is_wide();
            cols += if prev_was_wide { 2 } else { 1 };
        }

        (contents, prev_attrs, cols)
    }

    pub fn contents_diff(
        &self,
        prev: &Self,
        start: u16,
        width: u16,
        attrs: crate::attrs::Attrs,
    ) -> (Vec<u8>, crate::attrs::Attrs, u16) {
        let mut prev_was_wide = false;
        let mut skip = 0;
        let mut contents = vec![];
        let mut prev_attrs = attrs;
        let mut cols = 0;

        for (cell, prev_cell) in self
            .cells()
            .zip(prev.cells())
            .skip(start as usize)
            .take(width as usize)
        {
            if prev_was_wide {
                prev_was_wide = false;
                continue;
            }

            if cell == prev_cell {
                prev_was_wide = cell.is_wide();
                skip += if prev_was_wide { 2 } else { 1 };
            } else {
                if skip > 0 {
                    contents.extend(format!("\x1b[{}C", skip).as_bytes());
                    cols += skip;
                    skip = 0;
                }

                let attrs = cell.attrs();
                if &prev_attrs != attrs {
                    contents.append(&mut attrs.escape_code_diff(&prev_attrs));
                    prev_attrs = *attrs;
                }

                contents.extend(if cell.has_contents() {
                    cell.contents().as_bytes()
                } else {
                    b"\x1b[X\x1b[C"
                });

                prev_was_wide = cell.is_wide();
                cols += if prev_was_wide { 2 } else { 1 };
            }
        }

        (contents, prev_attrs, cols)
    }

    fn content_width(&self, start: u16) -> u16 {
        for (col, cell) in
            self.cells.iter().skip(start as usize).enumerate().rev()
        {
            if cell.has_contents()
                || cell.bgcolor() != crate::attrs::Color::Default
            {
                let width: u16 = col.try_into().unwrap();
                return width + 1;
            }
        }
        0
    }
}

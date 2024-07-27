pub struct Input {
    buf: String,
    character_index: usize,
}

impl Input {
    pub const fn new() -> Self {
        Self {
            buf: String::new(),
            character_index: 0,
        }
    }

    pub fn get_index(&self) -> usize {
        self.character_index
    }

    pub fn get_string(&self) -> &String {
        &self.buf
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.buf.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn byte_index(&mut self) -> usize {
        self.buf
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.buf.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.buf.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.buf.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.buf = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.buf.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.buf.clear();
        self.character_index = 0;
    }
}

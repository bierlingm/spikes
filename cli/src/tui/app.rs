use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::spike::{Rating, Spike};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
}

pub struct App {
    pub spikes: Vec<Spike>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter_text: String,
    pub filter_rating: Option<Rating>,
    pub show_detail: bool,
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub table_offset: usize,
}

impl App {
    pub fn new(spikes: Vec<Spike>) -> Self {
        let filtered: Vec<usize> = (0..spikes.len()).collect();
        Self {
            spikes,
            filtered,
            selected: 0,
            filter_text: String::new(),
            filter_rating: None,
            show_detail: false,
            should_quit: false,
            input_mode: InputMode::Normal,
            table_offset: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::Filter => self.handle_filter_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
            }
            KeyCode::Enter => {
                self.show_detail = !self.show_detail;
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Filter;
            }
            KeyCode::Char('1') => {
                self.set_rating_filter(Some(Rating::Love));
            }
            KeyCode::Char('2') => {
                self.set_rating_filter(Some(Rating::Like));
            }
            KeyCode::Char('3') => {
                self.set_rating_filter(Some(Rating::Meh));
            }
            KeyCode::Char('4') => {
                self.set_rating_filter(Some(Rating::No));
            }
            KeyCode::Char('0') => {
                self.set_rating_filter(None);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Tab => {
                self.input_mode = InputMode::Filter;
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected = 0;
                self.table_offset = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                if !self.filtered.is_empty() {
                    self.selected = self.filtered.len() - 1;
                }
            }
            KeyCode::PageDown => {
                self.page_down(10);
            }
            KeyCode::PageUp => {
                self.page_up(10);
            }
            _ => {}
        }
    }

    fn handle_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Tab => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.filter_text.pop();
                self.apply_filters();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char(c) => {
                self.filter_text.push(c);
                self.apply_filters();
            }
            _ => {}
        }
    }

    fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected < self.filtered.len() - 1 {
            self.selected += 1;
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn page_down(&mut self, amount: usize) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + amount).min(self.filtered.len() - 1);
        }
    }

    fn page_up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
    }

    fn set_rating_filter(&mut self, rating: Option<Rating>) {
        self.filter_rating = rating;
        self.apply_filters();
    }

    fn apply_filters(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();

        self.filtered = self
            .spikes
            .iter()
            .enumerate()
            .filter(|(_, spike)| {
                if let Some(ref rating) = self.filter_rating {
                    if spike.rating.as_ref() != Some(rating) {
                        return false;
                    }
                }

                if !filter_lower.is_empty() {
                    let matches = spike.page.to_lowercase().contains(&filter_lower)
                        || spike.reviewer.name.to_lowercase().contains(&filter_lower)
                        || spike.comments.to_lowercase().contains(&filter_lower)
                        || spike.id.to_lowercase().contains(&filter_lower);
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .map(|(i, _)| i)
            .collect();

        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn selected_spike(&self) -> Option<&Spike> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.spikes.get(idx))
    }

}

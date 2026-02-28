use app::App;
use ratatui::{prelude::*, Terminal};

use crate::git_provider::Repository;

pub trait Searchable: Clone {
    fn display_label(&self) -> String;
}

impl Searchable for Repository {
    fn display_label(&self) -> String {
        self.to_rel_path().display().to_string()
    }
}

#[derive(Clone)]
pub struct StringItem(pub String);

impl Searchable for StringItem {
    fn display_label(&self) -> String {
        self.0.clone()
    }
}

pub struct Interactive {
    app: &'static crate::app::App,
}

impl Interactive {
    pub fn new(app: &'static crate::app::App) -> Self {
        Self { app }
    }

    pub fn interactive_search(
        &mut self,
        repositories: &[Repository],
    ) -> anyhow::Result<Option<Repository>> {
        self.interactive_search_items(repositories)
    }

    pub fn interactive_search_items<T: Searchable>(
        &mut self,
        items: &[T],
    ) -> anyhow::Result<Option<T>> {
        let backend = TermwizBackend::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let terminal = Terminal::new(backend)?;

        App::new(self.app, items).run(terminal)
    }
}

pub trait InteractiveApp {
    fn interactive(&self) -> Interactive;
}

impl InteractiveApp for &'static crate::app::App {
    fn interactive(&self) -> Interactive {
        Interactive::new(self)
    }
}

mod app {
    use crossterm::event::KeyModifiers;
    use ratatui::{
        crossterm::event::{self, Event, KeyCode},
        layout::{Constraint, Layout},
        prelude::TermwizBackend,
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{ListItem, ListState, Paragraph, StatefulWidget},
        Frame, Terminal,
    };

    use crate::fuzzy_matcher::FuzzyMatcherApp;

    use super::Searchable;

    pub struct App<'a, T: Searchable> {
        app: &'static crate::app::App,
        items: &'a [T],
        current_search: String,
        matched_items: Vec<T>,
        list: ListState,
    }

    impl<'a, T: Searchable> App<'a, T> {
        pub fn new(app: &'static crate::app::App, items: &'a [T]) -> Self {
            Self {
                app,
                items,
                current_search: String::default(),
                matched_items: Vec::default(),
                list: ListState::default(),
            }
        }

        fn update_matched_items(&mut self) {
            let labels: Vec<String> = self.items.iter().map(|i| i.display_label()).collect();
            let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

            let matched_keys = self
                .app
                .fuzzy_matcher()
                .match_pattern(&self.current_search, &label_refs);

            self.matched_items = matched_keys
                .into_iter()
                .filter_map(|key| {
                    self.items
                        .iter()
                        .find(|i| i.display_label() == key)
                        .cloned()
                })
                .collect();

            if self.list.selected().is_none() {
                self.list.select_first();
            }
        }

        pub fn run(
            mut self,
            mut terminal: Terminal<TermwizBackend>,
        ) -> anyhow::Result<Option<T>> {
            self.update_matched_items();

            loop {
                terminal.draw(|frame| self.draw(frame))?;

                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('c') = key.code
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(None);
                    }

                    match key.code {
                        KeyCode::Char(letter) => {
                            self.current_search.push(letter);
                            self.update_matched_items();
                        }
                        KeyCode::Backspace => {
                            if !self.current_search.is_empty() {
                                let _ =
                                    self.current_search.remove(self.current_search.len() - 1);
                                self.update_matched_items();
                            }
                        }
                        KeyCode::Esc => {
                            return Ok(None);
                        }
                        KeyCode::Enter => {
                            if let Some(selected) = self.list.selected()
                                && let Some(item) =
                                    self.matched_items.get(selected).cloned()
                            {
                                terminal.resize(ratatui::layout::Rect::ZERO)?;
                                return Ok(Some(item));
                            }

                            return Ok(None);
                        }
                        KeyCode::Up => self.list.select_next(),
                        KeyCode::Down => self.list.select_previous(),
                        _ => {}
                    }
                }
            }
        }

        fn draw(&mut self, frame: &mut Frame) {
            let vertical = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]);
            let [list_area, input_area] = vertical.areas(frame.area());

            let display_items: Vec<String> =
                self.matched_items.iter().map(|i| i.display_label()).collect();

            let list_items: Vec<ListItem> =
                display_items.into_iter().map(ListItem::from).collect();

            let list = ratatui::widgets::List::new(list_items)
                .direction(ratatui::widgets::ListDirection::BottomToTop)
                .scroll_padding(3)
                .highlight_symbol("> ")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
                .highlight_style(Style::default().bold().white());

            StatefulWidget::render(list, list_area, frame.buffer_mut(), &mut self.list);

            let input = Paragraph::new(Line::from(vec![
                Span::from("> ").blue(),
                Span::from(self.current_search.as_str()),
                Span::from(" ").on_white(),
            ]));

            frame.render_widget(input, input_area);
        }
    }
}

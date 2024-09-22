use app::App;

use crate::git_provider::Repository;

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
        let terminal = ratatui::init();
        let app_result = App::new(self.app, repositories).run(terminal);
        ratatui::restore();

        app_result
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
    use ratatui::{
        crossterm::event::{self, Event, KeyCode},
        layout::{Constraint, Layout},
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{ListItem, ListState, Paragraph, StatefulWidget},
        DefaultTerminal, Frame,
    };

    use crate::{
        commands::root::RepositoryMatcher, fuzzy_matcher::FuzzyMatcherApp, git_provider::Repository,
    };

    pub struct App<'a> {
        app: &'static crate::app::App,
        repositories: &'a [Repository],
        current_search: String,
        matched_repos: Vec<Repository>,
        list: ListState,
    }

    impl<'a> App<'a> {
        pub fn new(app: &'static crate::app::App, repositories: &'a [Repository]) -> Self {
            Self {
                app,
                repositories,
                current_search: String::default(),
                matched_repos: Vec::default(),
                list: ListState::default(),
            }
        }

        fn update_matched_repos(&mut self) {
            let res = self
                .app
                .fuzzy_matcher()
                .match_repositories(&self.current_search, self.repositories);

            //res.reverse();

            self.matched_repos = res;

            if self.list.selected().is_none() {
                self.list.select_first();
            }
        }

        pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<Option<Repository>> {
            self.update_matched_repos();

            loop {
                terminal.draw(|frame| self.draw(frame))?;

                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char(letter) => {
                            self.current_search.push(letter);
                            self.update_matched_repos();
                        }
                        KeyCode::Backspace => {
                            if !self.current_search.is_empty() {
                                let _ = self.current_search.remove(self.current_search.len() - 1);
                                self.update_matched_repos();
                            }
                        }
                        KeyCode::Esc => return Ok(None),
                        KeyCode::Enter => {
                            if let Some(selected) = self.list.selected() {
                                if let Some(repo) = self.matched_repos.get(selected).cloned() {
                                    return Ok(Some(repo));
                                }
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
            let [repository_area, input_area] = vertical.areas(frame.area());

            let repos = &self.matched_repos;

            let repo_items = repos
                .iter()
                .map(|r| r.to_rel_path().display().to_string())
                .collect::<Vec<_>>();

            let repo_list_items = repo_items
                .into_iter()
                .map(ListItem::from)
                .collect::<Vec<_>>();

            let repo_list = ratatui::widgets::List::new(repo_list_items)
                .direction(ratatui::widgets::ListDirection::BottomToTop)
                .scroll_padding(3)
                .highlight_symbol("> ")
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
                .highlight_style(Style::default().bold().white());

            StatefulWidget::render(
                repo_list,
                repository_area,
                frame.buffer_mut(),
                &mut self.list,
            );

            let input = Paragraph::new(Line::from(vec![
                Span::from("> ").blue(),
                Span::from(self.current_search.as_str()),
                Span::from(" ").on_white(),
            ]));

            frame.render_widget(input, input_area);
        }
    }
}

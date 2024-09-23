use std::{
    io::{stderr, Write},
    time::Duration,
};

use anyhow::Context;
use crossterm::{
    event::{EventStream, KeyCode, KeyEventKind},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use futures::{FutureExt, StreamExt};
use ratatui::{
    crossterm,
    prelude::*,
    widgets::{Block, Padding, Paragraph},
    TerminalOptions, Viewport,
};

use crate::components::{BatchCommand, Command};

use super::{
    create_dispatch,
    spinner::{Spinner, SpinnerState},
    Dispatch, IntoCommand, Msg, Receiver,
};

pub struct InlineCommand {
    spinner: SpinnerState,
    heading: String,
}

impl InlineCommand {
    pub fn new(heading: impl Into<String>) -> Self {
        Self {
            spinner: SpinnerState::default(),
            heading: heading.into(),
        }
    }

    pub async fn execute<F, Fut>(&mut self, func: F) -> anyhow::Result<()>
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: futures::Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        tracing::trace!("starting inline terminal");

        let mut terminal = ratatui::init_with_options(TerminalOptions {
            viewport: Viewport::Inline(3),
        });

        let (dispatch, mut receiver) = create_dispatch();
        let mut event_stream = crossterm::event::EventStream::new();
        let guard = TerminalGuard;

        tokio::spawn({
            let dispatch = dispatch.clone();

            async move {
                match func().await {
                    Ok(_) => dispatch.send(Msg::Success),
                    Err(e) => dispatch.send(Msg::Failure(e.to_string())),
                }
            }
        });

        loop {
            if self
                .update(&mut terminal, &dispatch, &mut receiver, &mut event_stream)
                .await?
            {
                terminal.draw(|f| {
                    let buf = f.buffer_mut();
                    buf.reset();
                })?;

                break;
            }
        }

        drop(guard);

        println!();

        Ok(())
    }

    async fn update(
        &mut self,
        terminal: &mut ratatui::Terminal<impl Backend>,
        dispatch: &Dispatch,
        receiver: &mut Receiver,
        event_stream: &mut EventStream,
    ) -> anyhow::Result<bool> {
        let input_event = event_stream.next().fuse();
        let next_msg = receiver.next().fuse();

        const FRAMES_PER_SECOND: f32 = 60.0;
        const TICK_RATE: f32 = 20.0;

        let period_frame = Duration::from_secs_f32(1.0 / FRAMES_PER_SECOND);
        let mut interval_frames = tokio::time::interval(period_frame);
        let period_tick = Duration::from_secs_f32(1.0 / TICK_RATE);
        let mut interval_ticks = tokio::time::interval(period_tick);

        let msg = tokio::select! {
            _ = interval_frames.tick() => {
                terminal.draw(|frame| self.draw(frame))?;
                None
            }
            _ = interval_ticks.tick() => {
                Some(Msg::Tick)
            }
            msg = next_msg => {
                msg
            }
            input = input_event => {
                if let Some(Ok(input)) = input {
                    self.handle_key_event(input)
                } else {
                    None
                }
            }
        };

        if let Some(msg) = msg {
            if Msg::Quit == msg {
                return Ok(true);
            }

            let mut cmd = self.update_state(&msg);

            loop {
                let msg = cmd.into_command().execute(dispatch);

                match msg {
                    Some(Msg::Quit) => return Ok(true),
                    Some(msg) => {
                        cmd = self.update_state(&msg);
                    }
                    None => break,
                }
            }
        }

        Ok(false)
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let spinner = Spinner::new(Span::from(&self.heading));

        let block = Block::new().padding(Padding::symmetric(2, 1));

        StatefulWidget::render(
            spinner.block(block),
            frame.area(),
            frame.buffer_mut(),
            &mut self.spinner,
        );
    }

    fn handle_key_event(&mut self, event: crossterm::event::Event) -> Option<Msg> {
        if let crossterm::event::Event::Key(key) = event {
            return match key.code {
                KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Char('c') => Some(Msg::Quit),
                _ => None,
            };
        }

        None
    }

    fn update_state(&mut self, msg: &Msg) -> impl IntoCommand {
        tracing::debug!("handling message: {:?}", msg);

        let mut batch = BatchCommand::default();

        match msg {
            Msg::Quit => {}
            Msg::Tick => {}
            Msg::Success => return Msg::Quit.into_command(),
            Msg::Failure(f) => {
                tracing::error!("command failed: {}", f);
                return Msg::Quit.into_command();
            }
        }

        batch.with(self.spinner.update(msg));

        batch.into_command()
    }
}

#[derive(Default)]
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        tracing::trace!("restoring inline terminal");
        ratatui::restore();
    }
}

use std::sync::Arc;

use futures::StreamExt;
use gpui::{AppContext, Context, Model, ModelContext, Subscription};
use serde::Deserialize;
use util::ResultExt;

use crate::{
    static_runnable_file::{Definition, RunnableProvider},
    RunState, Runnable, Source, StaticRunner, Token,
};
use futures::channel::mpsc::UnboundedReceiver;

pub struct StaticSource {
    runnables: Vec<Token>,
    _subscription: Subscription,
}

/// A Wrapper around deserializable T that keeps track of it's contents
/// via a provided channel. Once T value changes, the observers of [`TrackedFile`] are
/// notified.
pub struct TrackedFile<T> {
    parsed_contents: T,
}

impl<T: for<'a> Deserialize<'a> + PartialEq + 'static> TrackedFile<T> {
    pub fn new(
        initial_contents: T,
        mut tracker: UnboundedReceiver<String>,
        cx: &mut AppContext,
    ) -> Model<Self> {
        cx.new_model(move |cx| {
            cx.spawn(|this, mut cx| async move {
                while let Some(new_contents) = tracker.next().await {
                    let Some(new_contents) = serde_json::from_str(&new_contents).log_err() else {
                        continue;
                    };
                    this.update(&mut cx, |this: &mut TrackedFile<T>, cx| {
                        if this.parsed_contents != new_contents {
                            this.parsed_contents = new_contents;
                            cx.notify();
                        };
                    })?;
                }
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
            Self {
                parsed_contents: initial_contents,
            }
        })
    }

    fn get(&self) -> &T {
        &self.parsed_contents
    }
}

impl StaticSource {
    pub fn new(
        definitions: Model<TrackedFile<RunnableProvider>>,
        cx: &mut AppContext,
    ) -> Model<Box<dyn Source>> {
        cx.new_model(|cx| {
            let _subscription = cx.observe(&definitions, |this, new_definitions, cx| {
                let runnables = new_definitions
                    .read(cx)
                    .get()
                    .runnables
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(id, runnable)| token_from_definition(id, runnable, cx))
                    .collect();
                if let Some(this) = this.as_any().downcast_mut::<Self>() {
                    this.runnables = runnables;
                    cx.notify();
                }
            });
            Box::new(Self {
                runnables: Vec::new(),
                _subscription,
            })
        })
    }
}

impl Source for StaticSource {
    fn runnables_for_path(
        &mut self,
        _: &std::path::Path,
        _: &mut ModelContext<Box<dyn Source>>,
    ) -> Vec<Token> {
        self.runnables.clone()
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn token_from_definition(
    id: usize,
    runnable: Definition,
    cx: &mut ModelContext<Box<dyn Source>>,
) -> Token {
    let runner = StaticRunner::new(runnable.clone());
    let display_name = runner.name();
    let source = cx.weak_model();
    let state = cx.new_model(|_| RunState::NotScheduled(Arc::new(runner)));
    Token {
        id,
        metadata: Arc::new(crate::Metadata {
            source,
            display_name,
        }),
        state,
    }
}

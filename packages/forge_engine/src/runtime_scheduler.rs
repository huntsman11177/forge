use std::collections::VecDeque;

use serde_json::Value;

use crate::RuntimeContext;

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    StateChanged(String, Value),
    ProviderChanged(String, Value),
    ThemeApplied,
    Custom(String),
}

#[derive(Debug, Default)]
pub struct RuntimeScheduler {
    queue: VecDeque<RuntimeEvent>,
}

impl RuntimeScheduler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn schedule(&mut self, event: RuntimeEvent) {
        self.queue.push_back(event);
    }

    pub fn drain(&mut self) -> Vec<RuntimeEvent> {
        self.queue.drain(..).collect()
    }

    pub fn process<F>(&mut self, ctx: &mut RuntimeContext, mut handler: F)
    where
        F: FnMut(&RuntimeEvent, &mut RuntimeContext),
    {
        while let Some(event) = self.queue.pop_front() {
            handler(&event, ctx);
        }
    }
}

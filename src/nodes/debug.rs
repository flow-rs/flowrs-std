use std::{fmt::Debug, rc::Rc, sync::Arc};

use crate::{flow::app_state::FlowType, job::RuntimeConnectable, nodes::job::Node};

use super::{connection::Edge, job::Context};

pub struct DebugNode<I>
where
    I: Clone,
{
    name: String,
    state: Option<I>,
    props: I,
    context: Arc<Context>,

    pub input: Edge<I>,
    pub output: Option<Edge<I>>,
}

impl<I> DebugNode<I>
where
    I: Clone,
{
    pub fn new(name: &str, context: Arc<Context>, props: I) -> Self {
        Self {
            name: name.into(),
            state: None,
            props,
            context,
            input: Edge::new(),
            output: None,
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug,
{
    type Output = I;
    fn on_init(&mut self) {}

    fn on_ready(&mut self) {}

    fn on_shutdown(&mut self) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn update(&mut self) {
        if let Ok(input) = self.input.next_elem() {
            println!("{:?}", input);
        }
    }

    fn connect(&mut self, edge: Edge<I>) {
        self.output = Some(edge)
    }
}

impl<I: Clone + 'static> RuntimeConnectable for DebugNode<I> {
    fn input_at(&self, index: usize) -> FlowType {
        match index {
            0 => FlowType(Rc::new(self.input.clone())),
            _ => panic!("Intex out of bounds for DebugNode"),
        }
    }

    fn output_at(&self, index: usize) -> FlowType {
        match index {
            _ => panic!("Intex out of bounds for DebugNode"),
        }
    }
}

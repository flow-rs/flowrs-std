use std::fmt::{self, Debug};
use std::str::FromStr;

use flowrs::connection::EdgeTrait;
use flowrs::connection::{Input, Output};
use flowrs::exec::execution_mode::ExecutionMode;
use flowrs::node::{Node, UpdateError};
use flowrs::nodes::node_io::{
    NodeIO, SetupIO, SetupInputsSync, SetupOutputsSync, TypedInput, TypedOutput,
};
pub struct DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    io: NodeIO<(TypedInput<I>,), ()>,
}

impl<I> DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            io: NodeIO::new(
                (TypedInput {
                    input: Input::new_local(),
                },),
                (), // no outputs
            ),
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(Some(value)) = self.io.inputs.0.input.next() {
            println!(
                "[DebugNode] Thread {:?} | Value: {:?}",
                std::thread::current().id(),
                value
            );
        }
        Ok(())
    }

    fn set_execution_mode(&mut self, _mode: ExecutionMode) -> ExecutionMode {
        ExecutionMode::Continuous
    }

    fn get_execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Continuous
    }

    fn on_init(&mut self) -> Result<(), flowrs::node::InitError> {
        Ok(())
    }

    fn on_ready(&mut self) -> Result<(), flowrs::node::ReadyError> {
        Ok(())
    }

    fn on_shutdown(&mut self) -> Result<(), flowrs::node::ShutdownError> {
        Ok(())
    }

    fn get_input_count(&self) -> u128 {
        1
    }

    fn get_output_count(&self) -> u128 {
        0
    }

    fn setup_input(&mut self, idx: u128, local: bool) {
        self.io.inputs.setup_input_sync(idx, local);
    }

    fn setup_output(&mut self, idx: u128, _local: bool) {
        panic!(
            "[DebugNode] Unexpected setup_output call on node with no outputs (idx = {})",
            idx
        );
    }

    fn get_io_mut(&mut self) -> &mut dyn SetupIO {
        &mut self.io
    }
}

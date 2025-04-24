use std::fmt::Debug;
use std::str::FromStr;

use flowrs::connection::EdgeTrait;
use flowrs::connection::Input;
use flowrs::exec::execution_mode::ExecutionMode;
use flowrs::node::{Node, ReceiveError, UpdateError};
use flowrs::nodes::node_io::{NodeIO, SetupIO, SetupInputsSync, TypedInput};
pub struct DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    io: NodeIO<(TypedInput<I>,), ()>,
    warn_if_no_message: bool,
}

impl<I> DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    pub fn new(warn_if_no_message: bool) -> Self {
        Self {
            io: NodeIO::new(
                (TypedInput {
                    input: Input::new_local(),
                },),
                (), // no outputs
            ),
            warn_if_no_message,
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + FromStr + Send + Sync + 'static,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        match self.io.inputs.0.input.edge.take() {
            Some(value) => {
                tracing::debug!("[DebugNode] Value: {:?}", value);
            }
            None => {
                if self.warn_if_no_message {
                    tracing::debug!("[DebugNode] ⚠️ No value received.");
                }
                return Ok(()); // Gracefully skip to the next loop iteration
            }
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

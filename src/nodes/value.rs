use std::fmt;
use std::str::FromStr;

use flowrs::connection::EdgeTrait;
use flowrs::nodes::node_io::NodeIO;
use flowrs::nodes::node_io::SetupIO;
use flowrs::nodes::node_io::SetupInputsSync;
use flowrs::nodes::node_io::SetupOutputs;
use flowrs::nodes::node_io::SetupOutputsSync;
use flowrs::nodes::node_io::TypedOutput;
//use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Output,
    node::{Node, ReadyError, UpdateError},
};
use tokio::runtime::Runtime;
//#[derive(RuntimeConnectable)]
pub struct ValueNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    value: O,

    //#[output]
    pub io: NodeIO<(), (TypedOutput<O>,)>,
}

impl<O> ValueNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    // pub fn new(value: O, change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         value,
    //         output: Output::new(change_observer),
    //     }
    // }
    pub fn new(value: O) -> Self {
        Self {
            value,
            io: NodeIO::new(
                (),
                (TypedOutput {
                    output: Output::new_local(),
                },),
            ),
        }
    }
}

impl<O> Node for ValueNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    fn on_ready(&mut self) -> Result<(), ReadyError> {
        self.io
            .outputs
            .0
            .output
            .send(self.value.clone())
            .map_err(|e| ReadyError::Other(e.into()))?;
        Ok(())
    }

    fn get_input_count(&self) -> u128 {
        0
    }

    fn get_output_count(&self) -> u128 {
        1
    }

    fn setup_input(&mut self, idx: u128, local: bool) {
        //self.io.inputs.setup_input_sync(idx, local)
    }

    fn setup_output(&mut self, idx: u128, local: bool) {
        let rt = Runtime::new().unwrap();
        rt.block_on(self.io.outputs.0.output.setup_output(idx, local));
    }

    fn get_io_mut(&mut self) -> &mut dyn SetupIO {
        &mut self.io
    }
}

//#[derive(RuntimeConnectable)]
pub struct ValueUpdateNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    value: O,

    //#[output]
    pub io: NodeIO<(), (TypedOutput<O>,)>,
}

impl<O> ValueUpdateNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    // pub fn new(value: O, change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         value,
    //         output: Output::new(change_observer),
    //     }
    // }
    pub fn new(value: O) -> Self {
        Self {
            value,
            io: NodeIO::new(
                (),
                (TypedOutput {
                    output: Output::new_local(),
                },),
            ),
        }
    }
}

impl<O> Node for ValueUpdateNode<O>
where
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        self.io.outputs.0.output.send(self.value.clone())?;
        Ok(())
    }

    fn get_input_count(&self) -> u128 {
        0
    }

    fn get_output_count(&self) -> u128 {
        1
    }

    fn setup_input(&mut self, _idx: u128, _local: bool) {
        //self.io.inputs.setup_input(idx, local)
    }

    fn setup_output(&mut self, idx: u128, local: bool) {
        let rt = Runtime::new().unwrap();
        rt.block_on(self.io.outputs.0.output.setup_output(idx, local));
    }

    fn get_io_mut(&mut self) -> &mut dyn SetupIO {
        &mut self.io
    }
}

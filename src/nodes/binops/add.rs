use super::binops::BinOpState;
//use crate::handle_sequentially;
use flowrs::nodes::node::ReceiveError;
use flowrs::nodes::node_io::{
    NodeIO, SetupIO, SetupInputsSync, SetupOutputsSync, TypedInput, TypedOutput,
};
use flowrs::{
    connection::{EdgeTrait, Input, Output},
    node::{Node, UpdateError},
};
use std::{fmt, ops::Add, str::FromStr};
//#[derive(RuntimeConnectable)]
pub struct AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I1: Add<I2, Output = O>,
    I1: Send + Sync + 'static,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    I2: Send + Sync + 'static,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    state: BinOpState<I1, I2>,

    io: NodeIO<(TypedInput<I1>, TypedInput<I2>), (TypedOutput<O>,)>,
}

impl<I1, I2, O> AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I1: Add<I2, Output = O>,
    I1: Send + Sync + 'static,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    I2: Send + Sync + 'static,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    // pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         state: BinOpState::None,
    //         input_1: Input::new(),
    //         input_2: Input::new(),
    //         output_1: Output::new(change_observer),
    //     }
    // }

    pub fn new() -> Self {
        Self {
            io: NodeIO::new(
                (
                    TypedInput {
                        input: Input::new_local(),
                    },
                    TypedInput {
                        input: Input::new_local(),
                    },
                ),
                (TypedOutput {
                    output: Output::new_local(),
                },),
            ),
            state: BinOpState::None,
        }
    }

    fn handle_1(&mut self, v: I1) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I1(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Addition should happen pairwise.".into(),
                })
            }
            BinOpState::I2(i) => {
                let out = v + i.clone();
                self.state = BinOpState::None;
                self.io.outputs.0.output.send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I1(v),
        }
        Ok(())
    }

    fn handle_2(&mut self, v: I2) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I2(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Addition should happen pairwise.".into(),
                })
            }
            BinOpState::I1(i) => {
                let out = i.clone() + v;
                self.state = BinOpState::None;
                self.io.outputs.0.output.send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I2(v),
        }
        Ok(())
    }
}

impl<I1, I2, O> Node for AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I1: Add<I2, Output = O>,
    I1: Send + Sync + 'static,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    I2: Send + Sync + 'static,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + Sync + 'static,
{
    fn on_update(&mut self) -> anyhow::Result<(), UpdateError> {
        // First input pass
        match self.state {
            BinOpState::I1(_) => match self.io.inputs.0.input.next() {
                Ok(Some(i2)) => self.handle_1(i2)?,
                Ok(None) => return Ok(()),
                Err(ReceiveError::NoMessageAvailable) => return Ok(()),
                Err(e) => {
                    return Err(UpdateError::RecvError {
                        message: e.to_string(),
                    })
                }
            },
            _ => match self.io.inputs.1.input.next() {
                Ok(Some(i1)) => self.handle_2(i1)?,
                Ok(None) => return Ok(()),
                Err(ReceiveError::NoMessageAvailable) => return Ok(()),
                Err(e) => {
                    return Err(UpdateError::RecvError {
                        message: e.to_string(),
                    })
                }
            },
        }

        // Second input pass
        match self.state {
            BinOpState::I1(_) => {
                match self.io.inputs.0.input.next() {
                    Ok(Some(i2)) => self.handle_1(i2)?,
                    _ => {} // Gracefully skip if not ready or error already handled
                }
            }
            _ => {
                match self.io.inputs.1.input.next() {
                    Ok(Some(i1)) => self.handle_2(i1)?,
                    _ => {} // Gracefully skip if not ready or error already handled
                }
            }
        }

        Ok(())
    }

    fn set_execution_mode(
        &mut self,
        _mode: flowrs::exec::execution_mode::ExecutionMode,
    ) -> flowrs::exec::execution_mode::ExecutionMode {
        flowrs::exec::execution_mode::ExecutionMode::Continuous
    }

    fn get_execution_mode(&self) -> flowrs::exec::execution_mode::ExecutionMode {
        flowrs::exec::execution_mode::ExecutionMode::Continuous
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

    // fn update_controller(&self) -> Option<Box<dyn flowrs::node::UpdateController>> {
    //     None
    // }

    fn get_input_count(&self) -> u128 {
        2 // Two inputs
    }

    fn get_output_count(&self) -> u128 {
        1 // One output
    }

    fn setup_input(&mut self, idx: u128, local: bool) {
        self.io.inputs.setup_input_sync(idx, local);
    }

    fn setup_output(&mut self, idx: u128, local: bool) {
        self.io.outputs.setup_output_sync(idx, local);
    }

    fn get_io_mut(&mut self) -> &mut dyn SetupIO {
        &mut self.io
    }
}

#[cfg(test)]
mod tests {
    use flowrs::{
        comm::{communication::NodeCommunicator, thread_communicator::ThreadCommunicator},
        node::UpdateError,
    };

    use super::*;

    #[test]
    fn should_add_132() -> Result<(), UpdateError> {
        // Create AddNode (using default local communication)
        let mut add: AddNode<i32, i32, i32> = AddNode::new();
        add.io.inputs.0.input.send(1)?;
        add.io.inputs.1.input.send(2)?;
        add.on_update()?;

        let expected = 3;
        let actual = add.io.outputs.0.output.next()?.unwrap();
        Ok(assert_eq!(expected, actual))
    }

    // #[test]
    // fn should_serialize_deserialize() -> Result<(), UpdateError> {
    //     let change_observer = ChangeObserver::new();

    //     let mut add: AddNode<i32, i32, i32> = AddNode::new(Some(&change_observer));
    //     add.input_1.send(2)?;
    //     add.on_update()?;

    //     let expected = r#"{"state":{"I1":2},"input_1":null,"input_2":null,"output_1":null}"#;
    //     let actual = serde_json::to_string(&add).unwrap();

    //     assert_eq!(expected, actual);

    //     let res = serde_json::from_str::<AddNode<i32, i32, i32>>(expected);
    //     let expected;
    //     match res {
    //         Ok(val) => expected = val,
    //         Err(e) => panic!("{}", e),
    //     }
    //     let actual = add.state;

    //     assert_eq!(
    //         serde_json::to_string(&expected.state).unwrap(),
    //         serde_json::to_string(&actual).unwrap()
    //     );
    //     Ok(())
    // }
}

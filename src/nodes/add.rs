use std::ops::Add;

use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, InitError, Node, ReadyError, ShutdownError, UpdateError},
};
use flowrs_derive::RuntimeConnectable;

#[derive(Clone)]
enum AddNodeState<I1, I2> {
    I1(I1),
    I2(I2),
    None,
}

#[derive(RuntimeConnectable)]
pub struct AddNode<I1, I2, O>
where
    I1: Clone,
    I2: Clone,
{
    
    state: AddNodeState<I1, I2>,

    #[input]
    pub input_1: Input<I1>,
    #[input]
    pub input_2: Input<I2>,
    #[output]
    pub output_1: Output<O>,
}

impl<I1, I2, O> AddNode<I1, I2, O>
where
    I1: Clone + Add<I2, Output = O> + Send,
    I2: Clone + Send,
    O: Clone + Send,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
           
            state: AddNodeState::None,
            input_1: Input::new(),
            input_2: Input::new(),
            output_1: Output::new(change_observer),
        }
    }

    fn handle_1(&mut self, v: I1) -> Result<(), UpdateError> {
        match &self.state {
            AddNodeState::I1(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Addition should happen pairwise.".into(),
                })
            }
            AddNodeState::I2(i) => {
                let out = v + i.clone();
                self.state = AddNodeState::None;
                // TODO replace match statement by ? one error handling is implemented
                match self.output_1.clone().send(out) {
                    Ok(_) => (),
                    Err(_) => return Err(UpdateError::ConnectError { message: "You attempted to send to an output where no succesor Node is connected.".into() }),
                };
            }
            AddNodeState::None => self.state = AddNodeState::I1(v),
        }
        Ok(())
    }

    fn handle_2(&mut self, v: I2) -> Result<(), UpdateError> {
        match &self.state {
            AddNodeState::I2(_) => {
                return Err(UpdateError::SequenceError {
                    
                    message: "Addition should happen pairwise.".into(),
                })
            }
            AddNodeState::I1(i) => {
                let out = i.clone() + v;
                self.state = AddNodeState::None;
                // TODO replace match statement by ? one error handling is implemented
                match self.output_1.clone().send(out) {
                    Ok(_) => (),
                    Err(_) => return Err(UpdateError::ConnectError { message: "You attempted to send to an output where no succesor Node is connected.".into() }),
                };
            }
            AddNodeState::None => self.state = AddNodeState::I2(v),
        }
        Ok(())
    }
}

impl<I1, I2, O> Node for AddNode<I1, I2, O>
where
    I1: Add<I2, Output = O> + Clone + Send,
    I2: Clone + Send,
    O: Clone + Send,
{

    // To be replaced by macro
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(i1) = self.input_1.next() {
            println!("UPDATE1");
            self.handle_1(i1)?;
        }

        if let Ok(i2) = self.input_2.next() {
            println!("UPDATE2");
            self.handle_2(i2)?;
        }
        Ok(())
    }
}

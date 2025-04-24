use flowrs::{
    connection::{connect, Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};

use flowrs::RuntimeConnectable;
use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct BinarySwitch<T> {
    #[output]
    pub output_1: Output<T>,

    #[output]
    pub output_2: Output<T>,

    #[input]
    pub input: Input<T>,

    switch: bool,
}

impl<T> BinarySwitch<T> {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output_1: Output::new(change_observer),
            output_2: Output::new(change_observer),
            input: Input::new(),
            switch: true,
        }
    }
}

impl<T> Node for BinarySwitch<T>
where
    T: Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next() {
            self.switch = !self.switch;
            if self.switch {
                self.output_1
                    .send(input)
                    .map_err(|e| UpdateError::Other(e.into()))?;
            } else {
                self.output_2
                    .send(input)
                    .map_err(|e| UpdateError::Other(e.into()))?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_switch() -> Result<(), UpdateError> {
    let n: i32 = 1024;
    let numbers: Vec<i32> = (0..n).collect();

    let mut switch_node: BinarySwitch<i32> = BinarySwitch::new(None);

    let e_1: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();
    let e_2: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();

    connect(switch_node.output_1.clone(), e_1.clone());
    connect(switch_node.output_2.clone(), e_2.clone());

    for i in 0..n {
        switch_node.input.send(i)?;
        switch_node.on_update()?;
    }

    let odd_numbers: Vec<_> = numbers.iter().filter(|&x| x % 2 != 0).cloned().collect();
    let even_numbers: Vec<_> = numbers.iter().filter(|&x| x % 2 == 0).cloned().collect();

    let mut odd_res_nums = Vec::new();
    while let Ok(v) = e_1.next() {
        odd_res_nums.push(v);
    }

    let mut even_res_nums = Vec::new();
    while let Ok(v) = e_2.next() {
        even_res_nums.push(v);
    }

    //tracing::debug!("{:?}", odd_res_nums);
    //tracing::debug!("{:?}", even_res_nums);

    assert_eq!(odd_res_nums, odd_numbers);
    assert_eq!(even_res_nums, even_numbers);
    Ok(())
}

#[test]
fn should_serialize_deserialize() -> Result<(), UpdateError> {
    let mut switch: BinarySwitch<i32> = BinarySwitch::new(None);

    let e_1: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();
    let e_2: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();

    connect(switch.output_1.clone(), e_1.clone());
    connect(switch.output_2.clone(), e_2.clone());

    switch.input.send(2)?;
    switch.on_update()?;

    let expected = r#"{"output_1":null,"output_2":null,"input":null,"switch":false}"#;
    let actual = serde_json::to_string(&switch).unwrap();

    assert_eq!(expected, actual);

    let res = serde_json::from_str::<BinarySwitch<i32>>(expected);
    let expected;
    match res {
        Ok(val) => expected = val,
        Err(e) => panic!("{}", e),
    }
    let actual = switch.switch;

    assert_eq!(
        serde_json::to_string(&expected.switch).unwrap(),
        serde_json::to_string(&actual).unwrap()
    );
    Ok(())
}

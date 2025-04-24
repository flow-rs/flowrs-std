use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};

use flowrs::RuntimeConnectable;

use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct ToJsonStringNode<T> {
    #[output]
    pub output: Output<String>,

    #[input]
    pub input: Input<T>,
}

impl<T> ToJsonStringNode<T>
where
    T: Serialize,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new(),
        }
    }
}

impl<T> Node for ToJsonStringNode<T>
where
    T: Serialize + Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(obj) = self.input.next() {
            let json_str = serde_json::to_string(&obj).map_err(|e| UpdateError::Other(e.into()))?;
            self.output
                .send(json_str)
                .map_err(|e| UpdateError::Other(e.into()))?;
        }
        Ok(())
    }
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct FromJsonStringNode<T> {
    #[output]
    pub output: Output<T>,

    #[input]
    pub input: Input<String>,
}

impl<T> FromJsonStringNode<T>
where
    T: for<'a> Deserialize<'a> + Send,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new(),
        }
    }
}

impl<T> Node for FromJsonStringNode<T>
where
    T: for<'a> Deserialize<'a> + Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(json_str) = self.input.next() {
            let obj = serde_json::from_str(json_str.as_str())
                .map_err(|e| UpdateError::Other(e.into()))?;
            self.output
                .send(obj)
                .map_err(|e| UpdateError::Other(e.into()))?;
        }
        Ok(())
    }
}

#[test]
fn test_to_json_and_from_string() -> Result<(), anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    struct TestStruct {
        a: i32,
        b: i32,
    }

    let inp = TestStruct { a: 42, b: -42 };

    tracing::debug!("Path: {}", module_path!());

    let mut to_string_node: ToJsonStringNode<TestStruct> = ToJsonStringNode::new(None);
    let mut from_string_node: FromJsonStringNode<TestStruct> = FromJsonStringNode::new(None);

    let e: flowrs::connection::Edge<TestStruct> = flowrs::connection::Edge::new();

    flowrs::connection::connect(
        to_string_node.output.clone(),
        from_string_node.input.clone(),
    );
    flowrs::connection::connect(from_string_node.output.clone(), e.clone());

    to_string_node.input.send(inp.clone())?;

    to_string_node.on_update()?;
    from_string_node.on_update()?;

    let out = e.next().expect("");

    assert_eq!(out, inp);
    Ok(())
}

#[test]
fn test_from_json_string_error() {
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    struct TestStruct {
        a: i32,
        b: i32,
    }

    let mut from_string_node: FromJsonStringNode<TestStruct> = FromJsonStringNode::new(None);

    let e: flowrs::connection::Edge<TestStruct> = flowrs::connection::Edge::new();

    flowrs::connection::connect(from_string_node.output.clone(), e.clone());

    let _ = from_string_node.input.send("{a:\"TEXT\"}}".to_string());

    let res = from_string_node.on_update();
    match res {
        Err(_) => assert!(true),
        Ok(_) => assert!(false),
    }
}

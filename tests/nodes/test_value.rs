#[cfg(test)]
mod nodes {
   
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::value::ValueNode;

    #[test]
    fn should_send_on_ready() -> Result<(), anyhow::Error> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 
        let mut node = ValueNode::new(42, Some(&change_observer));
        let mock_output = Edge::new();
        connect(node.output.clone(), mock_output.clone());
        let _ = node.on_ready();

        let expected = 42;
        let actual: i32 = mock_output.next()?.into();
        Ok(assert!(expected == actual))
    }
}

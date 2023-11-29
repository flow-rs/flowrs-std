use flowrs::node::{ ChangeObserver, Node, InitError};
use flowrs::connection::{Input, Output};
use flowrs::RuntimeConnectable;

use std::fs::File;

#[derive(RuntimeConnectable)]
pub struct DummyNode {
    name: String,

    #[input]
    pub input_1: Input<i32>,
    #[output]
    pub output_1: Output<i32>,
    err_on_init: bool
}

impl DummyNode {
    pub fn new(name: &str, err_on_init: bool, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            name: name.into(),
            input_1: Input::new(),
            output_1: Output::new(change_observer),
            err_on_init: err_on_init
        }
    }
}

impl Node for DummyNode {
    fn on_init(&mut self)-> Result<(), InitError> {
        
        if self.err_on_init {
            let _file = File::open("").map_err(|err| InitError::Other(err.into()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod sched {
    

    use std::{thread, sync::mpsc, time::Duration};
    use flowrs::{node::ChangeObserver, connection::{Input, connect}, flow_impl::Flow,  exec::{execution::{StandardExecutor, Executor}, node_updater::MultiThreadedNodeUpdater}, sched::round_robin::RoundRobinScheduler};

    use crate::sched::test_sched::DummyNode;

    #[test]
    fn test_executor() {
 
        let (sender, receiver) = mpsc::channel();
        
        let change_observer = ChangeObserver::new();  

        let n1: DummyNode = DummyNode::new("node_1", false, Some(&change_observer));
        let mock_input = Input::<i32>::new();        
        connect(n1.output_1.clone(), mock_input.clone());


        let mut flow: Flow = Flow::new_empty();

        let _ = n1.input_1.send(1);
      
        
        flow.add_node(n1);

        let thread_handle = thread::spawn( move || {
        
            let num_workers = 4;
            let mut executor = StandardExecutor::new(change_observer);
            
            let _ = sender.send(executor.controller());

            let _ = executor.run(flow, RoundRobinScheduler::new(), MultiThreadedNodeUpdater::new(num_workers));
        });

        let controller = receiver.recv().unwrap();

        thread::sleep(Duration::from_secs(3));

        println!("CANCEL");

        controller.lock().unwrap().cancel();
       
        thread_handle.join().unwrap();

        println!("DONE");


        //println!("Has next: {}",  mock_output.has_next());

    }


    #[test]
    fn test_error_behavior() {

        let change_observer = ChangeObserver::new();  

       let n1: DummyNode = DummyNode::new("node_1", true, Some(&change_observer));
       let n2: DummyNode = DummyNode::new("node_2", true, Some(&change_observer));
       let mut flow: Flow = Flow::new_empty();
      
       flow.add_node(n1);
       flow.add_node(n2);

       let mut ex = StandardExecutor::new(change_observer);

       match ex.run(flow, RoundRobinScheduler::new(), MultiThreadedNodeUpdater::new(1)) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true)
       }

    }
} 
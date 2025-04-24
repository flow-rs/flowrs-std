use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Input,
    node::{Node, UpdateError},
};
use std::sync::mpsc::Sender;

#[derive(RuntimeConnectable)]
pub struct ReportNode {
    pub sender: Sender<bool>,

    #[input]
    pub input: Input<bool>,
}

impl ReportNode {
    pub fn new(sender: Sender<bool>) -> Self {
        Self {
            sender: sender,
            input: Input::new(),
        }
    }
}

impl Node for ReportNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next() {
            let _res = self.sender.send(input);
        }
        Ok(())
    }
}

#[cfg(test)]
mod nodes {
    use std::{sync::mpsc::channel, task::Poll, thread, time::Duration};

    use flowrs::{
        connection::connect,
        exec::{
            execution::{Executor, StandardExecutor},
            node_updater::{MultiThreadedNodeUpdater, NodeUpdater, SingleThreadedNodeUpdater},
        },
        flow_impl::Flow,
        node::ChangeObserver,
        sched::round_robin::RoundRobinScheduler,
    };

    use flowrs_std::{
        debug::DebugNode,
        timer::{PollTimer, TimerNode, TimerNodeConfig, TimerStrategy, WaitTimer},
        value::ValueNode,
    };
    use serde::{Deserialize, Serialize};

    use crate::nodes::test_timer::ReportNode;

    fn timer_test_with<
        T: TimerStrategy<bool> + Send + 'static,
        U: NodeUpdater + Drop + Send + 'static,
    >(
        node_updater: U,
        timer: T,
    ) where
        T: Clone + Deserialize<'static> + Serialize,
    {
        let sleep_seconds = 5;
        let timer_interval_seconds = 1;

        let change_observer: ChangeObserver = ChangeObserver::new();
        let (sender, receiver) = channel::<bool>();

        let node_1 = ValueNode::new(
            TimerNodeConfig {
                duration: core::time::Duration::from_secs(timer_interval_seconds),
            },
            Some(&change_observer),
        );

        let node_2: TimerNode<T, bool> =
            TimerNode::new_with_token(timer, true, Some(&change_observer));

        let node_3 = DebugNode::<bool>::new(Some(&change_observer));

        let node_4 = ReportNode::new(sender);

        connect(node_1.output.clone(), node_2.config_input.clone());
        connect(node_2.token_output.clone(), node_3.input.clone());
        connect(node_3.output.clone(), node_4.input.clone());

        let mut flow: Flow = Flow::new_empty();
        flow.add_node(node_1);
        flow.add_node(node_2);
        flow.add_node(node_3);
        flow.add_node(node_4);

        let (controller_sender, controller_receiver) = channel();
        let thread_handle = thread::spawn(move || {
            let mut executor = StandardExecutor::new(change_observer);

            controller_sender
                .send(executor.controller())
                .expect("Controller sender cannot send.");

            executor
                .run(flow, RoundRobinScheduler::new(), node_updater)
                .expect("Run failed.");
        });

        let controller = controller_receiver.recv().unwrap();

        thread::sleep(Duration::from_secs(sleep_seconds));

        //tracing::debug!("                                      ----> {:?} CANCEL", std::thread::current().id());

        controller.lock().unwrap().cancel();

        thread_handle.join().unwrap();

        let num_iters = receiver.iter().count();

        let asserted_num_iters = sleep_seconds / timer_interval_seconds;

        //tracing::debug!("{} {}", num_iters, asserted_num_iters.abs_diff(num_iters as u64));
        assert!(asserted_num_iters.abs_diff(num_iters as u64) <= 1);
    }

    #[test]
    fn test() {
        timer_test_with(MultiThreadedNodeUpdater::new(4), WaitTimer::new(true));

        timer_test_with(MultiThreadedNodeUpdater::new(4), WaitTimer::new(false));

        timer_test_with(
            SingleThreadedNodeUpdater::new(Some(100)),
            WaitTimer::new(true),
        );

        timer_test_with(SingleThreadedNodeUpdater::new(Some(100)), PollTimer::new());

        // This combination cannot work, since the single execution thread is blocked by the timer.
        // timer_test_with(SingleThreadedNodeUpdater::new(), WaitTimer::new(false));

        // This combination cannot work, since with multiple workers, a the execution unit sleeps without written outputs.
        // timer_test_with(MultiThreadedNodeUpdater::new(4), TimeSliceTimer::new());
    }

    #[test]
    fn should_deserialize_serialize() {
        let change_observer: ChangeObserver = ChangeObserver::new();
        let timer = WaitTimer::new(true);
        let node = TimerNode::new_with_token(timer, Some(true), Some(&change_observer));

        let expected = r#"{"timer":{"own_thread":true,"_marker":null},"config_input":null,"token_input":null,"token_output":null,"token_object":true}"#;
        let actual = serde_json::to_string(&node).unwrap();

        assert_eq!(expected, actual);

        let res = serde_json::from_str::<TimerNode<WaitTimer<bool>, bool>>(expected);
        let expected;
        match res {
            Ok(val) => expected = val,
            Err(e) => panic!("{}", e),
        }
        let actual = node.config_input.clone();

        assert_eq!(
            serde_json::to_string(&expected.config_input.clone()).unwrap(),
            serde_json::to_string(&actual).unwrap()
        );

        let timer = PollTimer::new();
        let node = TimerNode::new_with_token(timer, Some(true), Some(&change_observer));

        let expected = r#"{"timer":{"every":{"secs":0,"nanos":0},"_marker":null},"config_input":null,"token_input":null,"token_output":null,"token_object":true}"#;
        let actual = serde_json::to_string(&node).unwrap();

        assert_eq!(expected, actual);

        let res = serde_json::from_str::<TimerNode<PollTimer<bool>, bool>>(expected);
        let expected;
        match res {
            Ok(val) => expected = val,
            Err(e) => panic!("{}", e),
        }
        let actual = node.config_input.clone();

        assert_eq!(
            serde_json::to_string(&expected.config_input.clone()).unwrap(),
            serde_json::to_string(&actual).unwrap()
        );
    }
}

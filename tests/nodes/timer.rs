#[cfg(test)]
mod nodes {
    use std::{sync::mpsc, thread, time::Duration};

    use flowrs::{
        connection::{connect, Edge},
        node::ChangeObserver,
        sched::{
            executor::{Executor, MultiThreadedExecutor},
            flow::Flow,
            scheduler::RoundRobinScheduler,
            version::Version,
        },
    };

    use flowrs_std::{
        debug::DebugNode,
        timer::{Timer, TimerNode, TimerNodeConfig, TimerNodeToken},
        value::ValueNode,
    };

    #[test]
    fn test() {
        let change_observer: ChangeObserver = ChangeObserver::new();

        let node_1 = ValueNode::new(
            "node_1",
            &change_observer,
            TimerNodeConfig {
                duration: core::time::Duration::from_secs(1),
            },
        );

        let node_2 = TimerNode::new("node_2", &change_observer, Timer::new(true));

        let node_3 = DebugNode::<TimerNodeToken>::new("node_3", &change_observer);

        let mock_input = Edge::new();

        connect(node_1.output.clone(), node_2.config_input.clone());
        connect(node_2.token_output.clone(), node_3.input.clone());
        connect(node_3.output.clone(), mock_input.clone());

        let mut flow = Flow::new("flow_1", Version::new(1, 0, 0), vec![]);
        flow.add_node(node_1);
        flow.add_node(node_2);
        flow.add_node(node_3);

        let (sender, receiver) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            let num_threads = 4;
            let mut executor = MultiThreadedExecutor::new(num_threads, change_observer);
            let mut scheduler = RoundRobinScheduler::new();

            let _ = sender.send(executor.controller());

            executor.run(flow, scheduler);
        });

        let controller = receiver.recv().unwrap();

        thread::sleep(Duration::from_secs(5));

        tracing::debug!("{:?} CANCEL", std::thread::current().id());

        controller.lock().unwrap().cancel();

        thread_handle.join().unwrap();

        /*
        node_1.on_ready();
        node_2.update();

        thread::sleep(Duration::from_secs(1));
        node_3.update();

        thread::sleep(Duration::from_secs(1));
        node_3.update();
         */
    }
}

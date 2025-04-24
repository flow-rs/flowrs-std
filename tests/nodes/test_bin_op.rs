#[cfg(test)]
mod nodes {
    use anyhow::Error;
    use flowrs::connection::{connect, Edge, Input, Output, RuntimeConnectable};
    use flowrs::node::{ChangeObserver, Node};
    use flowrs_std::add::AddNode;
    use flowrs_std::and::AndNode;
    use flowrs_std::div::DivNode;
    use flowrs_std::mul::MulNode;
    use flowrs_std::or::OrNode;
    use flowrs_std::sub::SubNode;
    use std::any::Any;
    use std::rc::Rc;
    use std::{thread, vec};

    /// Scenario:
    ///
    /// [0, 1, ..., 100]
    ///         \
    ///          >-<Add>--[100, 100, ..., 100]
    ///         /
    /// [100, 99, ..., 0]
    #[test]
    fn should_add_multiple_132_sequentially() -> Result<(), Error> {
        let change_observer = ChangeObserver::new();

        let mut add: AddNode<i32, i32, i32> = AddNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(add.output_1.clone(), mock_output.clone());
        (0..100).for_each(|int| {
            let _ = add.input_1.send(int);
        });
        (0..101).rev().for_each(|int| {
            let _ = add.input_2.send(int);
        });
        (0..100).for_each(|_| {
            let _ = add.on_update();
        });
        let mut actual = vec![];
        for _ in 0..100 {
            let curr = mock_output.next()?;
            actual.push(curr)
        }
        let exected = vec![100; 100];
        Ok(assert!(
            exected == actual,
            "expected was: {:?} while actual was {:?}",
            exected,
            actual
        ))
    }

    /// Scenario:
    ///
    /// [0, 1, ..., 100]
    ///         \
    ///          >-<Add1>----[100, 100, ..., 100]
    ///         /                     \
    /// [100, 99, ..., 0]              >-<Add2>--[101, 101, ..., 101]
    ///                               /
    ///                        [1, 1, ..., 1]
    #[test]
    fn should_add_multiple_132_parallel() -> Result<(), Error> {
        let change_observer: ChangeObserver = ChangeObserver::new();

        let mut add1 = AddNode::new(Some(&change_observer));
        let mut add2 = AddNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(add1.output_1.clone(), add2.input_1.clone());
        connect(add2.output_1.clone(), mock_output.clone());
        (0..100).for_each(|int| {
            let _ = add1.input_1.send(int);
        });
        (0..101).rev().for_each(|int| {
            let _ = add1.input_2.send(int);
        });
        (0..100).rev().for_each(|_| {
            let _ = add2.input_2.send(1);
        });

        let handle1 = thread::spawn(move || {
            (0..1000).for_each(|_| {
                match add1.on_update() {
                    Ok(_) => (),
                    Err(e) => tracing::debug!("{:?}", e),
                };
            });
        });
        let handle2 = thread::spawn(move || {
            (0..1000).for_each(|_| {
                match add2.on_update() {
                    Ok(_) => (),
                    Err(e) => tracing::debug!("{:?}", e),
                };
            });
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let mut actual = vec![];
        for _ in 0..100 {
            let res = mock_output.next();
            if let Ok(curr) = res {
                actual.push(curr)
            }
        }
        let exected = vec![101; 100];
        Ok(assert!(
            exected == actual,
            "expected was: {:?}, len {} while actual was {:?}, len {}",
            exected,
            exected.len(),
            actual,
            actual.len()
        ))
    }

    #[test]
    #[should_panic(expected = "Index 1 out of bounds for AddNode with output len 1.")]
    fn should_fail_on_output_out_of_bounds() {
        let change_observer: ChangeObserver = ChangeObserver::new();

        let add: AddNode<i32, i32, i32> = AddNode::new(Some(&change_observer));

        add.output_at(1);
    }

    macro_rules! should_accept_input {
    ($($name:ident: ($kind:ident<$ty:ident>, $value:expr),)*) => {
    $(
        #[test]
        fn $name() {
            let index = $value;
            let change_observer: ChangeObserver = ChangeObserver::new();
            let node: $kind<$ty, $ty, $ty> = $kind::new(Some(&change_observer));
            let input: Rc<dyn Any> = node.input_at(index);
            let input_downcasted = input.downcast::<Input<$ty>>();
            assert!(input_downcasted.is_ok())
        }
    )*
    }}

    macro_rules! should_reject_input {
    ($($name:ident: ($kind:ident<$ty:ident>, $value:expr),)*) => {
    $(
        #[test]
        #[should_panic]
        fn $name() {
            let index = $value;
            let change_observer: ChangeObserver = ChangeObserver::new();
            let node: $kind<$ty, $ty, $ty> = $kind::new(Some(&change_observer));
            node.input_at(index);
        }
    )*
    }}

    macro_rules! should_accept_output {
    ($($name:ident: ($kind:ident <$ty:ident>, $value:expr),)*) => {
    $(
        #[test]
        fn $name() {
            let index = $value;
            let change_observer: ChangeObserver = ChangeObserver::new();
            let node: $kind<$ty, $ty, $ty> = $kind::new(Some(&change_observer));
            let output: Rc<dyn Any> = node.output_at(index);
            let output_downcasted = output.downcast::<Output<$ty>>();
            assert!(output_downcasted.is_ok())
        }
    )*
    }}

    macro_rules! should_reject_output {
    ($($name:ident: ($kind:ident <$ty:ident>, $value:expr),)*) => {
    $(
        #[test]
        #[should_panic]
        fn $name() {
            let index = $value;
            let change_observer: ChangeObserver = ChangeObserver::new();
            let node: $kind<$ty, $ty, $ty> = $kind::new(Some(&change_observer));
            node.output_at(index);
        }
    )*
    }}

    should_accept_input! {
        add_input_0: (AddNode<f32>, 0),
        add_input_1: (AddNode<f32>, 1),
        mul_input_0: (MulNode<f32>, 0),
        mul_input_1: (MulNode<f32>, 1),
        sub_input_0: (SubNode<f32>, 0),
        sub_input_1: (SubNode<f32>, 1),
        div_input_0: (DivNode<f32>, 0),
        div_input_1: (DivNode<f32>, 1),
        and_input_0: (AndNode<u8>, 0),
        and_input_1: (AndNode<u8>, 1),
        or_input_0: (OrNode<u8>, 0),
        or_input_1: (OrNode<u8>, 1),
    }

    should_reject_input! {
        add_input_2: (AddNode<f32>, 2),
        mul_input_2: (MulNode<f32>, 2),
        div_input_2: (DivNode<f32>, 2),
        sub_input_2: (SubNode<f32>, 2),
        and_input_2: (AndNode<u8>, 2),
        or_input_2: (OrNode<u8>, 2),
    }

    should_accept_output! {
        add_output_0: (AddNode<f32>, 0),
        mul_output_0: (MulNode<f32>, 0),
        sub_output_0: (SubNode<f32>, 0),
        div_output_0: (DivNode<f32>, 0),
        and_output_0: (AndNode<u8>, 0),
        or_output_0: (OrNode<u8>, 0),
    }

    should_reject_output! {
        add_output_1: (AddNode<f32>, 1),
        mul_output_1: (MulNode<f32>, 1),
        div_output_1: (DivNode<f32>, 1),
        sub_output_1: (SubNode<f32>, 1),
        and_output_1: (AndNode<u8>, 1),
        or_output_1: (OrNode<u8>, 1),
    }
}

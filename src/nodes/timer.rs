use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateController, UpdateError},
};
use serde::{Deserialize, Serialize};

use std::{
    marker::PhantomData,
    sync::{Arc, Condvar, Mutex},
    thread,
};
use web_time::{Duration, Instant};

#[derive(Clone, Deserialize, Serialize)]
pub struct TimerNodeConfig {
    pub duration: Duration,
}

pub trait TimerStrategy<U> {
    fn start<F>(&mut self, every: Duration, closure: F)
    where
        F: 'static + FnMut() + Send;
    fn update(&mut self, _output: &mut Output<U>, token: U) {}
    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        None
    }
}

struct WaitTimerUpdateController {
    cond_var: Arc<(Mutex<bool>, Condvar)>,
}

impl WaitTimerUpdateController {
    pub fn new(cond_var: Arc<(Mutex<bool>, Condvar)>) -> Self {
        Self { cond_var: cond_var }
    }
}

impl UpdateController for WaitTimerUpdateController {
    fn cancel(&mut self) {
        let (lock, cvar) = &*self.cond_var;
        let mut run = lock.lock().unwrap();
        *run = false;
        cvar.notify_one();
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SelectedTimer<U> {
    #[cfg(not(target_arch = "wasm32"))]
    timer: WaitTimer<U>,
    #[cfg(target_arch = "wasm32")]
    timer: PollTimer<U>,
}

impl<U> TimerStrategy<U> for SelectedTimer<U> {
    fn start<F>(&mut self, every: Duration, mut closure: F)
    where
        F: 'static + FnMut() + Send,
    {
        self.timer.start(every, closure);
    }

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        self.timer.update_controller()
    }

    fn update(&mut self, output: &mut Output<U>, token: U) {
        self.timer.update(output, token);
    }
}

impl<U> SelectedTimer<U> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {
            timer: WaitTimer::new(true),
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Self {
        Self {
            timer: PollTimer::new(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct WaitTimer<U> {
    own_thread: bool,
    #[serde(skip)]
    cond_var: Arc<(Mutex<bool>, Condvar)>,
    _marker: PhantomData<U>,
}

impl<U> TimerStrategy<U> for WaitTimer<U> {
    fn start<F>(&mut self, every: Duration, mut closure: F)
    where
        F: 'static + FnMut() + Send,
    {
        let pair = self.cond_var.clone();

        let mut timer_closure = move || {
            loop {
                (closure)();

                let (lock, cvar) = &*pair;

                let result = cvar
                    .wait_timeout_while(lock.lock().unwrap(), every, |&mut run| run)
                    .unwrap();

                if !result.1.timed_out() {
                    //tracing::debug!("TIMER SHUTDOWN");
                    break;
                }
            }
        };

        if self.own_thread {
            thread::spawn(timer_closure);
        } else {
            (timer_closure)();
        }
    }

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        Some(Arc::new(Mutex::new(WaitTimerUpdateController::new(
            self.cond_var.clone(),
        ))))
    }
}

impl<U> WaitTimer<U> {
    pub fn new(own_thread: bool) -> Self {
        Self {
            own_thread: own_thread,
            cond_var: Arc::new((Mutex::new(true), Condvar::new())),
            _marker: PhantomData,
        }
    }
}

fn now() -> Instant {
    Instant::now()
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PollTimer<U> {
    every: Duration,
    #[serde(skip)]
    #[serde(default = "now")]
    last_tick: Instant,
    _marker: PhantomData<U>,
}

impl<U> PollTimer<U> {
    pub fn new() -> Self {
        Self {
            every: Duration::ZERO, //set later
            last_tick: Instant::now(),
            _marker: PhantomData,
        }
    }
}

impl<U> TimerStrategy<U> for PollTimer<U> {
    fn start<F>(&mut self, every: Duration, _closure: F)
    where
        F: 'static + FnMut() + Send,
    {
        self.every = every;
        self.last_tick = Instant::now();
    }

    fn update(&mut self, output: &mut Output<U>, token: U) {
        if self.last_tick.elapsed() >= self.every {
            let _res = output.send(token);
            self.last_tick = Instant::now();
            //tracing::debug!("TICK");
        }
    }
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct TimerNode<T, U>
where
    T: TimerStrategy<U>,
{
    timer: T,

    #[input]
    pub config_input: Input<TimerNodeConfig>,

    #[input]
    pub token_input: Input<U>,

    #[output]
    pub token_output: Output<U>,

    token_object: Option<U>,
}

impl<'a, T, U> TimerNode<T, U>
where
    T: Deserialize<'a> + Serialize + TimerStrategy<U>,
    U: Clone,
{
    pub fn new_with_token(
        timer: T,
        token_object: U,
        change_observer: Option<&ChangeObserver>,
    ) -> Self {
        Self {
            config_input: Input::new(),
            token_input: Input::new(),
            token_output: Output::new(change_observer),
            timer: timer,
            token_object: Some(token_object),
        }
    }

    pub fn new(timer: T, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            config_input: Input::new(),
            token_input: Input::new(),
            token_output: Output::new(change_observer),
            timer: timer,
            token_object: Option::None,
        }
    }
}

impl<'a, T, U> Node for TimerNode<T, U>
where
    T: Deserialize<'a> + Serialize + TimerStrategy<U> + Send,
    U: Clone + Send + Copy + 'static,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        // Try to get token object token input.
        if let Ok(token) = self.token_input.next() {
            self.token_object = Some(token);
        }

        // We have a token object.
        if let Some(token) = self.token_object {
            // If config changes, recreate timer.
            if let Ok(config) = self.config_input.next() {
                let mut token_output_clone = self.token_output.clone();
                let token_clone = token.clone();

                self.timer.start(config.duration, move || {
                    //tracing::debug!("                                                  {:?} TIMER TICK 1", std::thread::current().id());
                    let _res = token_output_clone.send(token_clone);
                    //tracing::debug!("                                                  {:?} TIMER TICK 2 {:?}", std::thread::current().id(), res);
                });
            }

            self.timer.update(&mut self.token_output, token);
            Ok(())
        } else {
            Err(UpdateError::Other(anyhow::Error::msg(
                "No token object to send.",
            )))
        }
    }

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        self.timer.update_controller()
    }
}

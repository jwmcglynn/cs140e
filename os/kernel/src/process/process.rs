use traps::TrapFrame;
use process::{State, Stack};
use std::mem;

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub trap_frame: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Option<Process> {
        let stack = Stack::new()?;

        Some(Process {
            trap_frame: Box::new(TrapFrame::default()),
            stack,
            state: State::Ready
        })
    }

    /// Gets the current process id from the processes trap frame.
    pub fn get_id(&self) -> u64 {
        self.trap_frame.tpidr
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occurred. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        match self.state {
            State::Waiting(_) => {
                // Use mem::replace to remove the value and work around the
                // borrow checker.
                let mut state = mem::replace(&mut self.state, State::Ready);
                let ready = if let State::Waiting(ref mut is_ready) = state {
                    is_ready(self)
                } else {
                    panic!("Invalid path");
                };

                if !ready {
                    self.state = state;
                }

                ready
            },
            State::Running => false,
            State::Ready => true
        }
    }
}

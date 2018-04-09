use traps::TrapFrame;
use SCHEDULER;
use pi::timer::current_time;
use process::State;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let start_time = current_time();
    let end_time = start_time + (ms as u64) * 1000;
    SCHEDULER.switch(State::Waiting(Box::new(move |p| {
        let now = current_time();
        if end_time <= now {
            p.trap_frame.x0 = (now - start_time) / 1000;
            p.trap_frame.x1_to_x29[6] = 0;
            true
        } else {
            false
        }
    })), tf).expect("Sleep has process");
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            sleep(tf.x0 as u32, tf);
        },
        _ => {
            // x7 = 1, syscall does not exist.
            tf.x1_to_x29[6] = 1;
        }
    }
}

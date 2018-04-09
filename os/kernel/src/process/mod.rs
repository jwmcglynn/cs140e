mod process;
mod state;
mod scheduler;
mod stack;

pub use self::process::{Process, Id};
pub use self::state::State;
pub use self::scheduler::{GlobalScheduler, TICK};
pub use self::stack::Stack;

pub fn sys_sleep(ms: u32) -> u32 {
    let error: u64;
    let result: u32;
    unsafe {
        asm!("mov x0, $2
              svc 1
              mov $0, x0
              mov $1, x7"
              : "=r"(result), "=r"(error)
              : "r"(ms)
              : "x0", "x7")
    }

    assert_eq!(error, 0);
    result
}

use std::process;
use std::ptr;

type Sighandler = extern "C" fn(i32);

#[repr(C)]
struct Sigaction {
    sa_handler: Sighandler,
    sa_mask: usize,
    sa_flags: usize,
}

extern "C" {
    fn sigaction(signum: i32, act: *const Sigaction, oldact: *mut Sigaction) -> i32;
}

extern "C" fn handle_exit(_signum: i32) {
    process::exit(0);
}

pub fn init() {
    let sa = Sigaction {
        sa_handler: handle_exit,
        sa_mask: 0,
        sa_flags: 0,
    };

    unsafe {
        sigaction(2, &sa, ptr::null_mut()); // SIGINT
        sigaction(15, &sa, ptr::null_mut()); // SIGTERM
    }
}

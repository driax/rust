// error-pattern:explicit failure

// This time we're testing that the stack limits are restored
// correctly after calling into the C stack and unwinding.
// See the hack in upcall_call_shim_on_c_stack where it messes
// with the stack limit.

use std;

extern mod rustrt {
    fn last_os_error() -> str;
}

fn getbig_call_c_and_fail(i: int) {
    if i != 0 {
        getbig_call_c_and_fail(i - 1);
    } else {
        rustrt::last_os_error();
        fail;
    }
}

class and_then_get_big_again {
  let x:int;
  new(x:int) {self.x = x;}
  drop {
    fn getbig(i: int) {
        if i != 0 {
            getbig(i - 1);
        }
    }
    getbig(10000);
  }
}

fn main() {
    do task::spawn {
        let r = and_then_get_big_again(4);
        getbig_call_c_and_fail(10000);
    };
}
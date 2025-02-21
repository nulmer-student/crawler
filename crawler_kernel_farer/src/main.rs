mod interface;
mod compile;
mod data;
mod intern;

use std::sync::Arc;
use crawler::run;
use crawler::interface::*;
use interface::KernelFaRer;

fn main() {
    let interface: AnyInterface = Arc::new(KernelFaRer {});
    run(interface);
}

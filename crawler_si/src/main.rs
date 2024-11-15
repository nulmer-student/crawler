mod si;

use std::sync::Arc;

use crawler::run;
use crawler::interface::*;
use si::FindVectorSI;

fn main() {
    // Run the crawler with the SI interface
    let interface: AnyInterface = Arc::new(FindVectorSI {});
    run(interface);
}

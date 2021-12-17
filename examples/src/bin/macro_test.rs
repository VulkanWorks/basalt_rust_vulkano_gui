extern crate basalt_common;
#[macro_use]
extern crate basalt_macros;

use basalt_common::*;

fn main() {
    let style = style! {
        position: window,
        pos_from_t: 100 pct,
        width: 100.0 px,
        position: {
            top: 5 px,
            bottom: 5,
        },
    };

    println!("{:?}", style);
}

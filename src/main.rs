#![feature(rustc_private)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;

use rustc_middle::ty::TyCtxt;
use rustc_public::CrateDef;
use std::ops::ControlFlow;

mod analyze_fn_def;

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run_with_tcx!(&rustc_args, run);
}

fn run(tcx: TyCtxt) -> ControlFlow<(), ()> {
    use std::io::Write;
    let stdout = &mut std::io::stdout();

    let local_crate = rustc_public::local_crate();
    for fn_def in local_crate.fn_defs() {
        if let Some(body) = fn_def.body() {
            _ = writeln!(stdout, "{:?}: ", fn_def.name());
            let colloctor = analyze_fn_def::collect(&body);
            for ty in &colloctor.v_ty {
                _ = writeln!(stdout, "  [ty] {ty:?}");
            }
            for place in &colloctor.v_place {
                _ = writeln!(
                    stdout,
                    "  [place] {place:?}\n    [ty] {:?}\n    [proj] {:?}",
                    place.ty(body.locals()).unwrap(),
                    place.projection
                );
            }
        }
    }
    ControlFlow::Break(())
}

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
mod fn_info;

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
            let name = fn_def.name();
            _ = writeln!(stdout, "\n{name}:");
            _ = body.dump(stdout, &name);
            let collector = analyze_fn_def::collect(&body);
            let finfo = fn_info::FnInfo::new(fn_def, collector, &body);
            _ = writeln!(stdout, "{:#?}\n{:#?}", finfo.callees, &finfo.adts);
            // for ty in &colloctor.v_ty {
            //     _ = writeln!(stdout, "  [ty] {ty:?}");
            // }
            // for val in &colloctor.v_place {
            //     _ = writeln!(
            //         stdout,
            //         "  [place {}] {:?}\n    [ty] {}\n    [proj] {:?}",
            //         val.place.local,
            //         val.span.diagnostic(),
            //         val.ty(&body),
            //         val.place.projection
            //     );
            // }
        }
    }
    ControlFlow::Break(())
}

#![feature(rustc_private)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_public_bridge;

use rustc_middle::ty::TyCtxt;
use std::ops::ControlFlow;

mod analyze_fn_def;
mod info_adt;
mod info_fn;
mod output;

mod utils;
pub use utils::{FxIndexMap, FxIndexSet};

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run_with_tcx!(&rustc_args, run);
}

fn run(tcx: TyCtxt) -> ControlFlow<(), ()> {
    use std::io::Write;
    let stdout = &mut std::io::stdout();

    let local_crate = rustc_public::local_crate();
    let fn_defs = local_crate.fn_defs();

    let mut map_fn = FxIndexMap::with_capacity_and_hasher(fn_defs.len(), Default::default());

    for fn_def in fn_defs {
        if let Some(body) = fn_def.body() {
            // let name = fn_def.name();
            // _ = writeln!(stdout, "\n{name}:");
            // _ = body.dump(stdout, &name);
            let collector = analyze_fn_def::collect(&body);
            let finfo = info_fn::FnInfo::new(collector, &body);
            // _ = writeln!(stdout, "{:#?}\n{:#?}", finfo.callees, &finfo.adts);

            let out_func = output::Function::new(fn_def, &finfo, &body, tcx);
            serde_json::to_writer_pretty(&mut *stdout, &out_func).unwrap();
            _ = writeln!(stdout);

            map_fn.insert(fn_def, finfo);
        }
    }

    _ = writeln!(stdout);
    let map_adt = info_adt::adt_info(&map_fn);
    for (adt, adt_info) in &map_adt {
        let out_adt = output::Adt::new(adt, adt_info, tcx);
        serde_json::to_writer_pretty(&mut *stdout, &out_adt).unwrap();
        _ = writeln!(stdout);
    }

    ControlFlow::Break(())
}

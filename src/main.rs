#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_public_bridge;
extern crate rustc_span;

use rustc_middle::ty::TyCtxt;
use rustc_public::CrateDef;
use std::ops::ControlFlow;

mod adt;
mod analyze_fn_def;
mod info_adt;
mod info_fn;
mod output;

mod utils;
pub use utils::{FxIndexMap, FxIndexSet, ThinVec};

fn main() {
    let rustc_args: Vec<_> = std::env::args().collect();
    _ = rustc_public::run_with_tcx!(&rustc_args, run);
}

fn run(tcx: TyCtxt) -> ControlFlow<(), ()> {
    let local_crate = rustc_public::local_crate();
    let fn_defs = local_crate.fn_defs();

    let mut cache_adt = Default::default();
    let writer = output::Writer::new(&local_crate.name);
    let mut map_fn = FxIndexMap::with_capacity_and_hasher(fn_defs.len(), Default::default());

    for fn_def in fn_defs {
        if let Some(body) = fn_def.body() {
            let v_sp: ThinVec<_> = fn_def
                .all_tool_attrs()
                .iter()
                .flat_map(|attr| {
                    safety_parser::safety::parse_attr_and_get_properties(attr.as_str())
                })
                .collect();

            let collector = analyze_fn_def::collect(&body);
            let finfo = info_fn::FnInfo::new(collector, &body, v_sp, &mut cache_adt);

            let out_func = output::Function::new(fn_def, &finfo, &body, tcx);
            out_func.dump(&writer);

            map_fn.insert(fn_def, finfo);
        }
    }

    let map_adt = info_adt::adt_info(&map_fn);
    for (adt, adt_info) in &map_adt {
        let out_adt = output::Adt::new(adt, adt_info, tcx);
        out_adt.dump(&writer);
    }

    ControlFlow::Break(())
}

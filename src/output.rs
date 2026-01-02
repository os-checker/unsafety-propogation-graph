use crate::{
    info_adt::{Access as RawAccess, AdtInfo},
    info_fn::{Adt as RawAdt, FnInfo},
    utils::FxIndexMap,
};
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    CrateDef,
    mir::Body,
    rustc_internal::internal,
    ty::{FnDef, Span},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Function {
    pub name: String,
    pub callees: Vec<String>,
    pub adts: FxIndexMap<String, Vec<String>>,
    pub span: String,
    pub src: String,
    pub mir: String,
}

impl Function {
    pub fn new(fn_def: FnDef, info: &FnInfo, body: &Body, tcx: TyCtxt) -> Self {
        let name = fn_def.name();
        let [span, src] = span_to_src(body.span, tcx);
        let mir = {
            let mut buf = Vec::with_capacity(1024);
            _ = body.dump(&mut buf, &name);
            String::from_utf8(buf).unwrap_or_default()
        };
        Function {
            name,
            callees: info
                .callees
                .iter()
                .map(|instance| instance.name())
                .collect(),
            adts: info
                .adts
                .iter()
                .map(|(adt, locals)| {
                    (
                        adt.to_string(tcx),
                        locals.access.iter().map(|acc| format!("{acc:?}")).collect(),
                    )
                })
                .collect(),
            span,
            src,
            mir,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Adt {
    pub name: String,
    pub constructors: Vec<String>,
    pub access_self_as_arg: Access,
    pub access_self_as_locals: Access,
    pub access_field: Vec<Access>,
    pub span: String,
    pub src: String,
}

impl Adt {
    pub fn new(adt: &RawAdt, info: &AdtInfo, tcx: TyCtxt) -> Adt {
        let [span, src] = span_to_src(adt.def.span(), tcx);
        Adt {
            name: adt.def.name(),
            constructors: v_fn_name(&info.constructors),
            access_self_as_arg: Access::new(&info.as_argument),
            access_self_as_locals: Access::new(&info.otherwise),
            access_field: info.fields.iter().map(Access::new).collect(),
            span,
            src,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Access {
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub other: Vec<String>,
}

impl Access {
    fn new(raw: &RawAccess) -> Access {
        Access {
            read: v_fn_name(&raw.read),
            write: v_fn_name(&raw.write),
            other: v_fn_name(&raw.other),
        }
    }
}

fn v_fn_name(v: &[FnDef]) -> Vec<String> {
    v.iter().map(|c| c.name()).collect()
}

/// Span to string and source code.
fn span_to_src(span: Span, tcx: TyCtxt) -> [String; 2] {
    let span_str = span.diagnostic();

    let span = internal(tcx, span);
    let src_map = tcx.sess.source_map();

    let src = src_map.span_to_snippet(span).unwrap_or_default();

    [span_str, src]
}

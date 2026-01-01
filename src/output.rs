use crate::info_fn::FnInfo;
use crate::utils::FxIndexMap;
use rustc_middle::ty::TyCtxt;
use rustc_public::{CrateDef, mir::Body, rustc_internal::internal, ty::FnDef};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OutputFunction {
    pub name: String,
    pub callees: Vec<String>,
    pub adts: FxIndexMap<String, Vec<String>>,
    pub span: String,
    pub src: String,
    pub mir: String,
}

impl OutputFunction {
    pub fn new(fn_def: FnDef, info: &FnInfo, body: &Body, tcx: TyCtxt) -> Self {
        let name = fn_def.name();
        let span = &body.span;
        let mir = {
            let mut buf = Vec::with_capacity(1024);
            _ = body.dump(&mut buf, &name);
            String::from_utf8(buf).unwrap_or_default()
        };
        OutputFunction {
            name,
            callees: info
                .callees
                .iter()
                .map(|instance| instance.name())
                .collect(),
            adts: info
                .adts
                .iter()
                .map(|(adt, access)| {
                    (
                        adt.to_string(tcx),
                        access.iter().map(|acc| format!("{acc:?}")).collect(),
                    )
                })
                .collect(),
            span: span.diagnostic(),
            src: tcx
                .sess
                .source_map()
                .span_to_snippet(internal(tcx, span))
                .unwrap_or_default(),
            mir,
        }
    }
}

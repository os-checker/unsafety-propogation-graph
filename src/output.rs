use crate::{
    adt::Adt as RawAdt,
    info_adt::{Access as RawAccess, AdtInfo},
    info_fn::FnInfo,
    utils::FxIndexMap,
};
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    CrateDef, DefId,
    mir::{Body, Safety},
    rustc_internal::internal,
    ty::{FnDef, Span},
};
use rustc_span::def_id::DefId as IDefId;
use safety_parser::{
    configuration::Tag as TagSpec,
    safety::{PropertiesAndReason, Property},
};
use serde::Serialize;
use std::{fs, io, path::PathBuf};

#[derive(Debug, Serialize)]
pub struct Function {
    pub name: String,
    pub safe: bool,
    pub callees: Vec<String>,
    pub adts: FxIndexMap<String, Vec<String>>,
    pub path: Vec<String>,
    pub span: String,
    pub src: String,
    pub mir: String,
    pub doc: String,
    pub tags: Tags,
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
            safe: matches!(fn_def.fn_sig().value.safety, Safety::Safe),
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
            path: def_path(fn_def.def_id(), tcx),
            span,
            src,
            mir,
            doc: doc_string(fn_def.def_id(), tcx),
            tags: Tags::new(&info.v_sp),
        }
    }

    pub fn dump(&self, writer: &Writer) {
        writer.dump_json("function", &self.name, self);
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
    pub kind: String,
    pub doc_adt: String,
    pub variant_fields: FxIndexMap<String, VariantField>,
}

impl Adt {
    pub fn new(adt: &RawAdt, info: &AdtInfo, tcx: TyCtxt) -> Adt {
        let [span, src] = span_to_src(adt.def.span(), tcx);

        let kind = format!("{:?}", adt.def.kind());
        let doc_adt = doc_string(adt.def.def_id(), tcx);

        let mut variant_fields =
            FxIndexMap::with_capacity_and_hasher(adt.variant_fields.len(), Default::default());
        let adt_def = internal(tcx, adt.def);
        for vf in &*adt.variant_fields {
            let idx = format!("{:?}", vf.idx);
            let name = vf.name.to_string();
            let old = match (vf.idx.field, vf.idx.variant) {
                // unit struct: no fields
                (None, None) => break,
                // enum variant probably without fields
                (None, Some(variant_idx)) => {
                    let did = adt_def.variant(variant_idx.into()).def_id;
                    let doc = doc_string_internel_did(did, tcx);
                    variant_fields.insert(idx, VariantField { name, doc })
                }
                (Some(field_idx), None) => {
                    let variant = adt_def.variant(0u32.into());
                    let field = variant
                        .fields
                        .get(rustc_abi::FieldIdx::from_u32(field_idx))
                        .unwrap();
                    let doc = doc_string_internel_did(field.did, tcx);
                    variant_fields.insert(idx, VariantField { name, doc })
                }
                (Some(_), Some(_)) => {
                    let doc = String::new();
                    variant_fields.insert(idx, VariantField { name, doc })
                }
            };
            assert!(old.is_none(), "{adt_def:?}: {vf:?} has been inserted")
        }

        Adt {
            name: adt.to_string(tcx),
            constructors: v_fn_name(&info.constructors),
            access_self_as_arg: Access::new(&info.as_argument),
            access_self_as_locals: Access::new(&info.otherwise),
            access_field: info.fields.iter().map(Access::new).collect(),
            span,
            src,
            kind,
            doc_adt,
            variant_fields,
        }
    }

    pub fn dump(&self, writer: &Writer) {
        writer.dump_json("adt", &self.name, self);
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

#[derive(Debug, Serialize)]
pub struct VariantField {
    pub name: String,
    pub doc: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Tags {
    pub tags: Vec<Property>,
    pub spec: FxIndexMap<String, TagSpec>,
    pub docs: Vec<Box<str>>,
}

impl Tags {
    pub fn new(v_sp: &[PropertiesAndReason]) -> Self {
        let mut this = Self::default();
        for sp in v_sp {
            this.docs.push(sp.gen_hover_doc());
            this.tags.extend_from_slice(&sp.tags);
            for tag in &sp.tags {
                let name = tag.tag.name();
                if let Some(spec) = tag.tag.get_spec()
                    && this.spec.get(name).is_none()
                {
                    this.spec.insert(name.to_owned(), spec.clone());
                }
            }
        }
        this
    }
}

fn v_fn_name(v: &[FnDef]) -> Vec<String> {
    v.iter().map(|c| c.name()).collect()
}

fn def_path(def_id: DefId, tcx: TyCtxt) -> Vec<String> {
    let def_path = tcx.def_path(internal(tcx, def_id));
    let mut v_path = Vec::with_capacity(def_path.data.len() + 1);
    v_path.push(tcx.crate_name(def_path.krate).to_string());
    v_path.extend(def_path.data.iter().map(|d| d.as_sym(false).to_string()));
    v_path
}

/// Span to string and source code.
fn span_to_src(span: Span, tcx: TyCtxt) -> [String; 2] {
    let span_str = span.diagnostic();

    let span = internal(tcx, span);
    let src_map = tcx.sess.source_map();

    let src = src_map.span_to_snippet(span).unwrap_or_default();

    [span_str, src]
}

fn doc_string(def_id: DefId, tcx: TyCtxt) -> String {
    let did = internal(tcx, def_id);
    doc_string_internel_did(did, tcx)
}

fn doc_string_internel_did(did: IDefId, tcx: TyCtxt) -> String {
    use rustc_hir::Attribute;
    use rustc_hir::attrs::AttributeKind;
    use std::fmt::Write;

    let mut buf = String::new();
    for attr in tcx.get_all_attrs(did) {
        if let Attribute::Parsed(AttributeKind::DocComment { comment, .. }) = attr {
            _ = writeln!(&mut buf, "{comment}");
        }
    }
    buf
}

pub enum Writer {
    BaseDir(PathBuf),
    Stdout,
}

impl Writer {
    pub fn new(crate_name: &str) -> Self {
        match base_dir(crate_name) {
            Some(dir) => {
                match fs::create_dir_all(&dir) {
                    Ok(()) => (),
                    Err(err) if err.kind() == io::ErrorKind::AlreadyExists => (),
                    Err(err) => panic!("The directory {dir:?} is not created: {err}"),
                }
                Writer::BaseDir(dir)
            }
            None => Writer::Stdout,
        }
    }

    pub fn dump_json(&self, parent: &str, fname_stem: &str, data: &impl Serialize) {
        match self {
            Writer::BaseDir(dir) => {
                let parent = dir.join(parent);
                match fs::create_dir(&parent) {
                    Ok(()) => (),
                    Err(err) if err.kind() == io::ErrorKind::AlreadyExists => (),
                    Err(err) => panic!("The directory {dir:?} is not created: {err}"),
                }

                let mut file_path = parent.join(fname_stem);
                file_path.set_extension("json");

                let file = fs::File::create(&file_path).unwrap();
                serde_json::to_writer_pretty(file, data).unwrap();
            }
            Writer::Stdout => {
                use io::Write;
                let stdout = &mut io::stdout();
                _ = writeln!(stdout);
                serde_json::to_writer_pretty(&mut *stdout, data).unwrap();
                _ = writeln!(stdout);
            }
        }
    }
}

/// The base directory `$UPG_DIR/crate_name` to store JSONs data.
/// If the environment variable `UPG_DIR` is not set, data will be printed to stdout.
pub fn base_dir(crate_name: &str) -> Option<PathBuf> {
    std::env::var("UPG_DIR")
        .ok()
        .map(|dir| PathBuf::from(dir).join(crate_name))
}

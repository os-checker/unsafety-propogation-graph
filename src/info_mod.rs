use crate::FxIndexMap;
use rustc_hir::{ImplItemImplKind, ImplItemKind, ItemId, ItemKind, OwnerNode, Ty, def_id::DefId};
use rustc_middle::ty::{TyCtxt, TyKind};
use rustc_span::Ident;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Navigation {
    pub flatten: FlattenFreeItems,
    pub navi: Navi,
}

pub type FlattenFreeItems = Vec<Vec<DefPath>>;
pub type Navi = FxIndexMap<usize, Vec<usize>>;

fn to_navi(v_path: &mut FlattenFreeItems) -> Navi {
    v_path.sort_unstable();
    Navi::default()
}

pub fn mod_tree(tcx: TyCtxt) -> Navigation {
    let mut v_path = FlattenFreeItems::new();

    // Root module.
    v_path.push(vec![DefPath::crate_root(tcx)]);

    // Free items.
    for item_id in tcx.hir_free_items() {
        let item = tcx.hir_item(item_id);
        match &item.kind {
            ItemKind::Fn { ident, .. } => {
                push_plain_item_path(DefPathKind::Fn, ident, &item_id, tcx, &mut v_path);
            }
            ItemKind::Struct(ident, ..) => {
                push_plain_item_path(DefPathKind::Struct, ident, &item_id, tcx, &mut v_path);
            }
            ItemKind::Enum(ident, ..) => {
                push_plain_item_path(DefPathKind::Enum, ident, &item_id, tcx, &mut v_path);
            }
            ItemKind::Union(ident, ..) => {
                push_plain_item_path(DefPathKind::Union, ident, &item_id, tcx, &mut v_path);
            }
            ItemKind::Trait(_, _, _, ident, ..) => {
                push_plain_item_path(DefPathKind::TraitDecl, ident, &item_id, tcx, &mut v_path);
            }
            ItemKind::Impl(imp) => {
                for id in imp.items {
                    let assoc = tcx.hir_impl_item(*id);
                    if matches!(assoc.kind, ImplItemKind::Fn(..)) {
                        let mut implementor_path = DefPath::from_ty(imp.self_ty, tcx);
                        let fn_name = assoc.ident.as_str();
                        match assoc.impl_kind {
                            ImplItemImplKind::Inherent { .. } => {
                                implementor_path.push(DefPath::new(DefPathKind::AssocFn, fn_name));
                            }
                            ImplItemImplKind::Trait {
                                trait_item_def_id, ..
                            } => {
                                if let Ok(did) = trait_item_def_id {
                                    let mut trait_name = def_path(did, tcx);
                                    // Put SelfTy under trait path.
                                    trait_name.extend(std::mem::take(&mut implementor_path));
                                    implementor_path = trait_name;
                                    implementor_path
                                        .push(DefPath::new(DefPathKind::AssocFn, fn_name));
                                }
                            }
                        }
                        v_path.push(implementor_path);
                    }
                }
            }
            _ => (),
        }
    }

    let navi = to_navi(&mut v_path);
    Navigation {
        flatten: v_path,
        navi,
    }
}

fn push_plain_item_path(
    kind: DefPathKind,
    ident: &Ident,
    item_id: &ItemId,
    tcx: TyCtxt,
    v_path: &mut Vec<Vec<DefPath>>,
) {
    let mut path = vec![DefPath::new(kind, ident.as_str())];
    push_parent_paths(&mut path, item_id, tcx);
    v_path.push(path);
}

fn push_parent_paths(path: &mut Vec<DefPath>, item_id: &ItemId, tcx: TyCtxt) {
    for (_, owner_node) in tcx.hir_parent_owner_iter(item_id.hir_id()) {
        match owner_node {
            OwnerNode::Item(owner_item) => {
                if let ItemKind::Mod(mod_ident, _) = owner_item.kind {
                    path.push(DefPath::new(DefPathKind::Mod, mod_ident.as_str()));
                }
            }
            OwnerNode::Crate(_) => path.push(DefPath::crate_root(tcx)),
            _ => (),
        }
    }
    path.reverse();
}

#[derive(Debug, Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct DefPath {
    pub kind: DefPathKind,
    pub name: Box<str>,
}

impl DefPath {
    pub fn new<S: Into<Box<str>>>(kind: DefPathKind, name: S) -> Self {
        Self {
            kind,
            name: name.into(),
        }
    }

    pub fn from_ty(ty: &Ty, tcx: TyCtxt) -> Vec<Self> {
        let hir_id = ty.hir_id;
        let typ = tcx.type_of(hir_id.owner).skip_binder();
        if let TyKind::Adt(def, _) = typ.kind() {
            def_path(def.did(), tcx)
        } else {
            vec![Self::new(DefPathKind::SelfTy, typ.to_string())]
        }
    }

    fn crate_root(tcx: TyCtxt) -> Self {
        let crate_name = tcx.crate_name(rustc_span::def_id::CrateNum::ZERO);
        DefPath::new(DefPathKind::Mod, crate_name.as_str())
    }
}

/// ADT path can be `[Mod, Adt]` where Adt is one of Struct, Enum, and Union.
///
/// Function path is a tricky, because there are cases like
/// * `[Mod, Fn]` for a free function.
/// * `[Mod, Struct, AssocFn]` for an inherent function.
/// * `[Mod, Struct, ImplTrait, AssocFn]` for a trait function.
/// * `[Mod, TraitDecl, AssocFn]` for a trait function definition.
/// * `[SelfTy, AssocFn]` for an unusual associated function like `impl &Adt`.
/// * `[Mod, ImplTrait, SelfTy, AssocFn]` for an unusual trait function like `impl Trait for &Adt`,
///   `impl Trait for (Adt1, Adt2)`, `impl<T> Trait for T`, or even `impl<T: Trait> Trait for T::U`.
#[derive(Debug, Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum DefPathKind {
    Mod,
    Fn,
    AssocFn,
    Struct,
    Enum,
    Union,
    TraitDecl,
    SelfTy,
    ImplTrait,
}

fn def_path(did: DefId, tcx: TyCtxt) -> Vec<DefPath> {
    use rustc_hir::{def::DefKind, definitions::DefPathData};

    let default = || vec![DefPath::new(DefPathKind::SelfTy, tcx.def_path_str(did))];

    let def_kind = tcx.def_kind(did);
    let def_path_kind = match def_kind {
        DefKind::Struct => DefPathKind::Struct,
        DefKind::Enum => DefPathKind::Enum,
        DefKind::Union => DefPathKind::Union,
        // TraitDecl has been handled in ItemKind::Trait; and def_path is called in ItemKind::Impl.
        DefKind::Trait => DefPathKind::ImplTrait,
        _ => return default(),
    };

    let mut v_path = Vec::new();
    let mod_path = tcx.def_path(did);
    let crate_name = tcx.crate_name(mod_path.krate);
    v_path.push(DefPath::new(DefPathKind::Mod, crate_name.as_str()));
    for data in &mod_path.data {
        if let DefPathData::TypeNs(sym) = data.data {
            v_path.push(DefPath::new(DefPathKind::Mod, sym.as_str()));
        } else {
            unimplemented!("{data:?} is not a type namespace, check out {did:?}")
        }
    }

    let last_path_seg = v_path.last_mut().unwrap();
    last_path_seg.kind = def_path_kind;
    v_path
}

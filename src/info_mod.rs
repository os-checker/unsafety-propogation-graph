use std::mem;

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

pub type ItemPath = Vec<DefPath>;
pub type FlattenFreeItems = Vec<ItemPath>;
pub type Navi = FxIndexMap<usize, Vec<usize>>;

fn to_navi(v_path: &mut FlattenFreeItems, tcx: TyCtxt) -> Navi {
    #[derive(Debug, Default)]
    struct Meta {
        parent_paths: FxIndexMap<DefPathKind, ItemPath>,
        item_idx: usize,
    }
    let mut map_paths = FxIndexMap::<ItemPath, Meta>::with_capacity_and_hasher(
        v_path.len() * 3 / 2,
        Default::default(),
    );
    // Add root module path.
    let crate_root = DefPath::crate_root(tcx);
    map_paths.insert(vec![crate_root.clone()], Default::default());

    // Collect mod paths and move all paths to the map.
    // The idx will be backfilled once v_path is sorted, so 0 is fake here.
    for free_item in mem::take(v_path) {
        assert_eq!(
            free_item[0], crate_root,
            "{free_item:?} must start from {crate_root:?}"
        );

        // [..mod_sep, ...]
        let mod_sep = free_item
            .iter()
            .take_while(|p| p.kind == DefPathKind::Mod)
            .count();
        let len = free_item.len();
        assert_ne!(
            len, mod_sep,
            "free item {free_item:?} mustn't be a module item"
        );

        // Insert free item path with parent item path.
        let mod_path = &free_item[..mod_sep];
        let item_parent_path = &mut map_paths.entry(free_item.clone()).or_default().parent_paths;

        match free_item[mod_sep].kind {
            DefPathKind::AssocFn | DefPathKind::Mod => (),
            // Put the assoc item under an ADT or trait path.
            kind if mod_sep + 1 != len => {
                let parent_item_path = &free_item[..mod_sep + 1];
                item_parent_path.insert(kind, parent_item_path.to_owned());
            }
            // Put the ADT or trait under mod path.
            _ => _ = item_parent_path.insert(DefPathKind::Mod, mod_path.to_owned()),
        }

        // Insert current and parent module paths.
        let mut pos = mod_sep;
        while pos > 1 && !map_paths.contains_key(&mod_path[..pos]) {
            let current_mod = mod_path[..pos].to_owned();
            let mod_value = map_paths.entry(current_mod).or_default();
            let parent = mod_path[..pos - 1].to_owned();
            mod_value.parent_paths.insert(DefPathKind::Mod, parent);
            pos -= 1;
        }
    }

    // Sort all paths.
    map_paths.sort_unstable_keys();
    // Set idx by the order.
    for (idx, value) in map_paths.values_mut().enumerate() {
        value.item_idx = idx;
    }

    let mut navi = Navi::default();
    for current_meta in map_paths.values() {
        for parent_path in current_meta.parent_paths.values() {
            let parent_meta = map_paths.get(parent_path).unwrap();
            navi.entry(parent_meta.item_idx)
                .and_modify(|v| v.push(current_meta.item_idx))
                .or_insert_with(|| vec![current_meta.item_idx]);
        }
    }

    // Flatten all paths.
    v_path.extend(map_paths.into_keys());

    navi
}

pub fn mod_tree(tcx: TyCtxt) -> Navigation {
    let mut v_path = FlattenFreeItems::new();

    // Free items: those items may be inaccesible from user's perspective,
    // and item paths are as per source code definitions.
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
                                    trait_name.extend(mem::take(&mut implementor_path));
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

    let navi = to_navi(&mut v_path, tcx);
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

#[derive(Clone, Debug, Serialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
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
#[derive(Clone, Copy, Debug, Serialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
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

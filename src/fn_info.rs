extern crate rustc_public_bridge;
use crate::analyze_fn_def::Collector;
use rustc_data_structures::fx::{FxIndexMap, FxIndexSet};
use rustc_public::{
    mir::{Body, Mutability, ProjectionElem, mono::Instance},
    ty::{AdtDef, FnDef, GenericArgs, RigidTy, Ty, TyKind, VariantIdx},
};
use rustc_public_bridge::IndexedVal;

pub struct FnInfo {
    /// Basic fn info like name, signature, body are queried through def id.
    pub defid: FnDef,
    /// All types and places mentioned in the function.
    pub collector: Collector,
    /// Direct callees in the function. The order is decided by MirVisitor,
    /// and called functions is monomorphized.
    pub callees: FxIndexSet<Instance>,
    /// Direct adt places in the function. The adt is monomorphized.
    pub adts: FxIndexMap<Adt, FxIndexSet<AdtAccess>>,
}

impl FnInfo {
    pub fn new(defid: FnDef, collector: Collector, body: &Body) -> FnInfo {
        let mut callees = FxIndexSet::default();
        for ty in &collector.v_ty {
            if let RigidTy::FnDef(fn_def, args) = &ty.ty
                && let Ok(instance) = Instance::resolve(*fn_def, args)
            {
                callees.insert(instance);
            }
        }

        let mut adts = FxIndexMap::default();
        for place in &collector.v_place {
            if let Some(local_decl) = body.local_decl(place.place.local) {
                println!("[local {}] {:?}", place.place.local, local_decl.ty);
                push_adt(&local_decl.ty, &place.place.projection, &mut adts);
            }
        }

        FnInfo {
            defid,
            collector,
            callees,
            adts,
        }
    }
}

/// Add an adt access or adt variant access.
fn push_adt(ty: &Ty, proj: &[ProjectionElem], adts: &mut FxIndexMap<Adt, FxIndexSet<AdtAccess>>) {
    let TyKind::RigidTy(ty) = ty.kind() else {
        return;
    };

    match ty {
        RigidTy::Adt(def, args) => {
            let adt = Adt { def, args };
            let access = adts.entry(adt).or_default();
            match proj {
                [ProjectionElem::Deref, ProjectionElem::Field(idx, _), ..] => {
                    access.insert(AdtAccess::DerefVariant(VariantIdx::to_val(*idx)))
                }
                [ProjectionElem::Deref] => access.insert(AdtAccess::Deref),
                [] => access.insert(AdtAccess::Plain),
                _ => access.insert(AdtAccess::Unknown(proj.into())),
            };
        }
        RigidTy::Ref(_, ref_ty, mutability) => {
            let TyKind::RigidTy(RigidTy::Adt(def, args)) = ref_ty.kind() else {
                return;
            };
            let adt = Adt { def, args };
            let access = adts.entry(adt).or_default();
            match proj {
                [ProjectionElem::Field(idx, _), ..] => {
                    let var_idx = VariantIdx::to_val(*idx);
                    let acc = if matches!(mutability, Mutability::Mut) {
                        AdtAccess::MutRefField(var_idx)
                    } else {
                        AdtAccess::RefField(var_idx)
                    };
                    access.insert(acc);
                }
                [] => {
                    let acc = if matches!(mutability, Mutability::Mut) {
                        AdtAccess::MutRef
                    } else {
                        AdtAccess::Ref
                    };
                    access.insert(acc);
                }
                _ => push_adt(&ref_ty, proj, adts),
            }
        }
        RigidTy::Tuple(v) => v.iter().for_each(|ty| push_adt(ty, proj, adts)),
        RigidTy::Slice(ty) => push_adt(&ty, proj, adts),
        _ => (),
    }
}

/// Monomorphized adt.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Adt {
    def: AdtDef,
    args: GenericArgs,
}

/// Reference to rederence to the adt or its field.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AdtAccess {
    Ref,
    MutRef,
    Deref,
    Plain,
    RefField(VariantIdx),
    MutRefField(VariantIdx),
    DerefVariant(VariantIdx),
    PlainVariant(VariantIdx),
    Unknown(Box<[ProjectionElem]>),
}

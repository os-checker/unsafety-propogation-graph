use crate::analyze_fn_def::Collector;
use crate::utils::{FxIndexMap, FxIndexSet, SmallVec};
use rustc_middle::ty::TyCtxt;
use rustc_public::ty::GenericArgKind;
use rustc_public::{
    CrateDef,
    mir::{Body, Mutability, ProjectionElem, mono::Instance},
    rustc_internal::internal,
    ty::{AdtDef, GenericArgs, RigidTy, Ty, TyKind, VariantIdx},
};
use rustc_public_bridge::IndexedVal;
use std::fmt;

pub struct FnInfo {
    /// The owned return type.
    ///
    /// When the adt has nested type parameters, we try to extract all the adts
    /// from them, e.g. `Result<Struct, Error>` results in three adts `Result`,
    /// `Struct` and `Error`. Generics will be skipped.
    /// This helps determin what functions are constructors: if a function returns
    /// a Result above, it's considered to be a constructors for each adt mentioned.
    pub ret_adts: SmallVec<[Adt; 1]>,
    /// All types and places mentioned in the function.
    #[expect(unused)]
    pub collector: Collector,
    /// Direct callees in the function. The order is decided by MirVisitor,
    /// and called functions is monomorphized.
    pub callees: FxIndexSet<Instance>,
    /// Direct adt places in the function. The adt is monomorphized.
    pub adts: FxIndexMap<Adt, FxIndexSet<AdtAccess>>,
}

impl FnInfo {
    pub fn new(collector: Collector, body: &Body) -> FnInfo {
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
                // println!("[local {}] {:?}", place.place.local, local_decl.ty);
                push_adt(&local_decl.ty, &place.place.projection, &mut adts);
            }
        }

        let mut ret_adts = Default::default();
        flatten_adts(&body.ret_local().ty, &mut ret_adts);

        FnInfo {
            ret_adts,
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
                        AdtAccess::MutRefVariant(var_idx)
                    } else {
                        AdtAccess::RefVariant(var_idx)
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

/// Owned adt in the type.
/// FIXME: The implementation is naive at present, because arguments traversal
/// stops at an explicit reference or raw pointer. That means something like
/// `struct A<'a, T>(&'a T)` will be incorrectly treated as owned `A` and `T`
/// when `A` and `T` are concrete/rigid types.
fn flatten_adts(ty: &Ty, v: &mut SmallVec<[Adt; 1]>) {
    let TyKind::RigidTy(ty) = ty.kind() else {
        return;
    };

    match ty {
        RigidTy::Adt(def, args) => {
            v.push(Adt {
                def,
                args: args.clone(),
            });
            for arg in &args.0 {
                if let GenericArgKind::Type(ty) = arg {
                    flatten_adts(ty, v)
                }
            }
        }
        RigidTy::Array(ty, _) => flatten_adts(&ty, v),
        RigidTy::Tuple(v_ty) => v_ty.iter().for_each(|ty| flatten_adts(ty, v)),
        _ => (),
    }
}

/// Monomorphized adt.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Adt {
    pub def: AdtDef,
    pub args: GenericArgs,
}

impl Adt {
    pub fn to_string(&self, tcx: TyCtxt) -> String {
        let adt_name = self.def.name();
        let args = internal(tcx, &self.args);
        let args = if args.is_empty() {
            ""
        } else {
            &args.print_as_list()
        };
        format!("{adt_name}{args}")
    }
}

/// Reference to rederence to the adt or its field.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AdtAccess {
    Ref,
    MutRef,
    Deref,
    Plain,
    RefVariant(VariantIdx),
    MutRefVariant(VariantIdx),
    DerefVariant(VariantIdx),
    Unknown(Box<[ProjectionElem]>),
}

impl fmt::Debug for AdtAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ref => write!(f, "Ref"),
            Self::MutRef => write!(f, "MutRef"),
            Self::Deref => write!(f, "Deref"),
            Self::Plain => write!(f, "Plain"),
            Self::RefVariant(arg0) => f.debug_tuple("RefVariant").field(&arg0.to_index()).finish(),
            Self::MutRefVariant(arg0) => f
                .debug_tuple("MutRefVariant")
                .field(&arg0.to_index())
                .finish(),
            Self::DerefVariant(arg0) => f
                .debug_tuple("DerefVariant")
                .field(&arg0.to_index())
                .finish(),
            Self::Unknown(arg0) => f
                .debug_tuple("Unknown")
                .field(&format_args!("{arg0:?}"))
                .finish(),
        }
    }
}

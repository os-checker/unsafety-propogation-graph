use crate::adt::{Adt, AdtAccess, CacheAdt, LocalsAccess, VaraintFieldIdx, new_adt};
use crate::analyze_fn_def::Collector;
use crate::utils::{FxIndexMap, FxIndexSet, SmallVec, ThinVec};
use rustc_public::{
    mir::{Body, Mutability, ProjectionElem, mono::Instance},
    ty::{GenericArgKind, RigidTy, Ty, TyKind},
};
use safety_parser::safety::PropertiesAndReason;

pub struct FnInfo {
    /// The owned return type.
    ///
    /// When the adt has nested type parameters, we try to extract all the adts
    /// from them, e.g. `Result<Struct, Error>` results in three adts `Result`,
    /// `Struct` and `Error`. Generics will be skipped.
    /// This helps determin what functions are constructors: if a function returns
    /// a Result above, it's considered to be a constructors for each adt mentioned.
    pub ret_adts: SmallVec<[Adt; 1]>,
    /// The number of arguments this function takes.
    pub arg_count: usize,
    /// All types and places mentioned in the function.
    #[expect(unused)]
    pub collector: Collector,
    pub v_sp: ThinVec<PropertiesAndReason>,
    /// Direct callees in the function. The order is decided by MirVisitor,
    /// and called functions is monomorphized.
    pub callees: FxIndexSet<Instance>,
    /// Direct adt places in the function. The adt is monomorphized.
    pub adts: FxIndexMap<Adt, LocalsAccess>,
}

impl FnInfo {
    pub fn new(
        collector: Collector,
        body: &Body,
        v_sp: ThinVec<PropertiesAndReason>,
        cache: &mut CacheAdt,
    ) -> FnInfo {
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
            let local_idx = place.place.local;
            if let Some(local_decl) = body.local_decl(local_idx) {
                push_adt(
                    local_idx,
                    &local_decl.ty,
                    &place.place.projection,
                    &mut adts,
                    cache,
                );
            }
        }
        // Clean up indices.
        adts.values_mut().for_each(|l| l.deduplicate_indices());

        let mut ret_adts = Default::default();
        flatten_adts(&body.ret_local().ty, &mut ret_adts, cache);

        FnInfo {
            ret_adts,
            arg_count: body.arg_locals().len(),
            collector,
            v_sp,
            callees,
            adts,
        }
    }
}

/// Add an adt access or adt variant access.
fn push_adt(
    idx: usize,
    ty: &Ty,
    proj: &[ProjectionElem],
    adts: &mut FxIndexMap<Adt, LocalsAccess>,
    cache: &mut CacheAdt,
) {
    let TyKind::RigidTy(ty) = ty.kind() else {
        return;
    };

    match ty {
        RigidTy::Adt(def, args) => {
            let adt = new_adt(def, args, cache);
            let local = adts.entry(adt).or_default();
            local.locals.push(idx);
            // FIXME: ProjectionElem::Downcast(VariantIdx) should also be handled.
            match proj {
                [ProjectionElem::Deref, ProjectionElem::Field(idx, _), ..] => {
                    let value = AdtAccess::DerefVariantField(VaraintFieldIdx::new_field(*idx));
                    local.access.insert(value)
                }
                [ProjectionElem::Deref] => local.access.insert(AdtAccess::Deref),
                [] => local.access.insert(AdtAccess::Plain),
                _ => local.access.insert(AdtAccess::Unknown(proj.into())),
            };
        }
        RigidTy::Ref(_, ref_ty, mutability) => {
            let TyKind::RigidTy(RigidTy::Adt(def, args)) = ref_ty.kind() else {
                return;
            };
            let adt = new_adt(def, args, cache);
            let local = adts.entry(adt).or_default();
            local.locals.push(idx);
            match proj {
                [ProjectionElem::Field(idx, _), ..] => {
                    let field_idx = VaraintFieldIdx::new_field(*idx);
                    let acc = if matches!(mutability, Mutability::Mut) {
                        AdtAccess::MutRefVariantField(field_idx)
                    } else {
                        AdtAccess::RefVariantField(field_idx)
                    };
                    local.access.insert(acc);
                }
                [] => {
                    let acc = if matches!(mutability, Mutability::Mut) {
                        AdtAccess::MutRef
                    } else {
                        AdtAccess::Ref
                    };
                    local.access.insert(acc);
                }
                _ => push_adt(idx, &ref_ty, proj, adts, cache),
            }
        }
        RigidTy::Tuple(v) => v.iter().for_each(|ty| push_adt(idx, ty, proj, adts, cache)),
        RigidTy::Slice(ty) => push_adt(idx, &ty, proj, adts, cache),
        _ => (),
    }
}

/// Owned adt in the type.
/// FIXME: The implementation is naive at present, because arguments traversal
/// stops at an explicit reference or raw pointer. That means something like
/// `struct A<'a, T>(&'a T)` will be incorrectly treated as owned `A` and `T`
/// when `A` and `T` are concrete/rigid types.
fn flatten_adts(ty: &Ty, v: &mut SmallVec<[Adt; 1]>, cache: &mut CacheAdt) {
    let TyKind::RigidTy(ty) = ty.kind() else {
        return;
    };

    match ty {
        RigidTy::Adt(def, args) => {
            v.push(new_adt(def, args.clone(), cache));
            for arg in &args.0 {
                if let GenericArgKind::Type(ty) = arg {
                    flatten_adts(ty, v, cache)
                }
            }
        }
        RigidTy::Array(ty, _) => flatten_adts(&ty, v, cache),
        RigidTy::Tuple(v_ty) => v_ty.iter().for_each(|ty| flatten_adts(ty, v, cache)),
        _ => (),
    }
}

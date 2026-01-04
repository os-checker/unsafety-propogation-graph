use crate::{
    adt::{Adt, AdtAccess},
    info_fn::FnInfo,
    utils::{FxIndexMap, ThinVec},
};
use rustc_public::ty::FnDef;

pub fn adt_info(map_fn: &FxIndexMap<FnDef, FnInfo>) -> FxIndexMap<Adt, AdtInfo> {
    let mut map_adt =
        FxIndexMap::<Adt, AdtInfo>::with_capacity_and_hasher(map_fn.len(), Default::default());

    for (&fn_def, fn_info) in map_fn {
        // Append the fn_def to adt map.
        for (adt, locals) in &fn_info.adts {
            let adt_info = map_adt.entry(adt.clone()).or_default();

            for access in &locals.access {
                let v = adt_info.map.entry(access.clone()).or_default();
                v.push(FnDefAdt {
                    fn_def,
                    as_argument: locals.is_argument(fn_info.arg_count),
                });
            }
        }

        // Append the constructor for adt.
        for adt in &fn_info.ret_adts {
            let adt_info = map_adt.entry(adt.clone()).or_default();
            adt_info.constructors.push(fn_def);
        }
    }

    // Initialize the rest fields.
    for (adt, info) in &mut map_adt {
        info.init(adt);
    }

    map_adt
}

#[derive(Debug, Default)]
pub struct AdtInfo {
    /// The variant access appear in user functions.
    pub map: FxIndexMap<AdtAccess, ThinVec<FnDefAdt>>,
    /// Functions in the form of `fn(...) -> Self`.
    pub constructors: ThinVec<FnDef>,
    /// Functions that access the whole adt appearing as arguments.
    /// Like `fn(&self)`, `fn(Self)`, ....
    pub as_argument: Access,
    /// Functions that access the whole adt otherwise (probably as plain locals).
    pub otherwise: Access,
    /// Functions that access the fields. The slice index corresponds to the field index.
    /// If the adt is not a struct, or unit struct (struct without field), the slices is empty.
    pub fields: Box<[Access]>,
}

impl AdtInfo {
    /// The function initializes the rest fields when `map` is ready.
    fn init(&mut self, adt: &Adt) {
        // Initialize field access.
        self.fields = adt
            .num_fields()
            .map(|len| vec![Access::default(); len].into())
            .unwrap_or_default();

        // Backfill access to adt and fields.
        for (access, v_fn) in &self.map {
            let push = |as_arg: &mut ThinVec<FnDef>, other: &mut ThinVec<FnDef>| {
                for f in v_fn {
                    if f.as_argument {
                        as_arg.push(f.fn_def);
                    } else {
                        other.push(f.fn_def);
                    }
                }
            };
            match access {
                AdtAccess::Ref => push(&mut self.as_argument.read, &mut self.otherwise.read),
                AdtAccess::MutRef | AdtAccess::Deref => {
                    push(&mut self.as_argument.write, &mut self.otherwise.write)
                }
                AdtAccess::Plain | AdtAccess::Unknown(_) => {
                    push(&mut self.as_argument.other, &mut self.otherwise.other)
                }
                AdtAccess::RefVariantField(idx) => {
                    if let Some(idx) = idx.field_idx() {
                        self.fields[idx].read = v_fn.iter().map(|f| f.fn_def).collect();
                    }
                }
                AdtAccess::MutRefVariantField(idx) | AdtAccess::DerefVariantField(idx) => {
                    if let Some(idx) = idx.field_idx() {
                        self.fields[idx].write.extend(v_fn.iter().map(|f| f.fn_def));
                    }
                }
            }
        }

        // Extract adts from type parameter.
    }
}

#[derive(Debug)]
pub struct FnDefAdt {
    pub fn_def: FnDef,
    pub as_argument: bool,
}

/// Access a place w.r.t the adt or field.
#[derive(Clone, Debug, Default)]
pub struct Access {
    /// Functions that only read the place via Ref or RefField.
    /// FIXME: Interior mutability is not handled yet.
    pub read: ThinVec<FnDef>,
    /// Functions that can write the place via MutRef, Deref, MutRefField, or DerefVariant.
    pub write: ThinVec<FnDef>,
    /// Functions that in other ways access the place, like Plain or Unknown.
    pub other: ThinVec<FnDef>,
}

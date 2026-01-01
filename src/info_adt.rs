use crate::info_fn::{Adt, AdtAccess, FnInfo};
use crate::utils::{FxIndexMap, ThinVec};
use rustc_public::ty::FnDef;
use rustc_public_bridge::IndexedVal;

pub fn adt_info(map_fn: &FxIndexMap<FnDef, FnInfo>) -> FxIndexMap<Adt, AdtInfo> {
    let mut map_adt =
        FxIndexMap::<Adt, AdtInfo>::with_capacity_and_hasher(map_fn.len(), Default::default());

    for (&fn_def, fn_info) in map_fn {
        // Append the fn_def to adt map.
        for (adt, access) in &fn_info.adts {
            let adt_info = map_adt.entry(adt.clone()).or_default();

            for acc in access {
                let v = adt_info.map.entry(acc.clone()).or_default();
                v.push(fn_def);
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
    pub map: FxIndexMap<AdtAccess, ThinVec<FnDef>>,
    /// Functions in the form of `fn(...) -> Self`.
    pub constructors: ThinVec<FnDef>,
    /// Functions that access the whole adt.
    pub this: Access,
    /// Functions that access the fields. The slice index corresponds to the field index.
    /// If the adt is not a struct, or unit struct (struct without field), the slices is empty.
    pub fields: Box<[Access]>,
}

impl AdtInfo {
    /// The function initializes the rest fields when `map` is ready.
    fn init(&mut self, adt: &Adt) {
        // Initialize field access.
        self.fields = vec![Access::default(); adt.def.num_variants()].into();

        // Backfill access to adt and fields.
        for (access, v_fn) in &self.map {
            match access {
                AdtAccess::Ref => self.this.read = v_fn.as_slice().into(),
                AdtAccess::MutRef | AdtAccess::Deref => self.this.write.extend_from_slice(v_fn),
                AdtAccess::Plain | AdtAccess::Unknown(_) => self.this.other.extend_from_slice(v_fn),
                AdtAccess::RefVariant(idx) => {
                    self.fields[idx.to_index()].read = v_fn.as_slice().into()
                }
                AdtAccess::MutRefVariant(idx) | AdtAccess::DerefVariant(idx) => {
                    self.fields[idx.to_index()].write.extend_from_slice(v_fn);
                }
            }
        }

        // Extract adts from type parameter.
    }
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

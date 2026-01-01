use crate::FxIndexMap;
use crate::info_fn::{Adt, AdtAccess, FnInfo};
use rustc_public::ty::FnDef;

pub fn adt_info(map_fn: &FxIndexMap<FnDef, FnInfo>) -> FxIndexMap<Adt, AdtInfo> {
    let mut map_adt =
        FxIndexMap::<Adt, AdtInfo>::with_capacity_and_hasher(map_fn.len(), Default::default());

    for (fn_def, fn_info) in map_fn {
        for (adt, access) in &fn_info.adts {
            let adt_info = map_adt.entry(adt.clone()).or_default();

            for acc in access {
                let v = adt_info.map.entry(acc.clone()).or_default();
                v.push(*fn_def);
            }
        }
    }

    map_adt
}

#[derive(Debug, Default)]
pub struct AdtInfo {
    pub map: FxIndexMap<AdtAccess, Vec<FnDef>>,
}

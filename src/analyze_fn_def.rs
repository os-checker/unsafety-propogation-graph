use rustc_data_structures::fx::FxHashSet;
use rustc_public::{
    mir::{
        Body, Place,
        visit::{Location, MirVisitor},
    },
    ty::{RigidTy, Ty, TyKind},
};

pub fn collect(body: &Body) -> Collector {
    let mut collect_types = Collector::default();
    collect_types.visit_body(body);
    collect_types
}

#[derive(Default)]
pub struct Collector {
    pub v_ty: Vec<RigidTy>,
    pub v_place: Vec<Place>,
}

impl MirVisitor for Collector {
    fn visit_ty(&mut self, ty: &Ty, _location: Location) {
        if let TyKind::RigidTy(ty) = ty.kind() {
            self.v_ty.push(ty);
        }
        self.super_ty(ty);
    }

    fn visit_place(
        &mut self,
        place: &Place,
        ptx: rustc_public::mir::visit::PlaceContext,
        location: Location,
    ) {
        self.v_place.push(place.clone());
        self.super_place(place, ptx, location);
    }
}

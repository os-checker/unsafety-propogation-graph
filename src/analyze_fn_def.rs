use crate::utils::ThinVec;
use rustc_public::{
    CrateDef,
    mir::{
        Body, Mutability, Place,
        visit::{Location, MirVisitor},
    },
    ty::{RigidTy, Span, Ty, TyKind},
};
use std::fmt::{self, Debug};

pub fn collect(body: &Body) -> Collector {
    let mut collect_types = Collector::default();
    collect_types.visit_body(body);
    collect_types
}

#[derive(Default)]
pub struct Collector {
    pub v_ty: ThinVec<Type>,
    pub v_place: ThinVec<Place2>,
}

impl MirVisitor for Collector {
    fn visit_ty(&mut self, ty: &Ty, location: Location) {
        if let TyKind::RigidTy(ty) = ty.kind() {
            self.v_ty.push(Type {
                ty,
                span: location.span(),
            });
        }
        self.super_ty(ty);
    }

    fn visit_place(
        &mut self,
        place: &Place,
        ptx: rustc_public::mir::visit::PlaceContext,
        location: Location,
    ) {
        self.v_place.push(Place2 {
            place: place.clone(),
            span: location.span(),
        });
        self.super_place(place, ptx, location);
    }
}

pub struct Type {
    pub ty: RigidTy,
    pub span: Span,
}

impl Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Type")
            .field("ty", &ty_str(&self.ty))
            .field("span", &self.span.diagnostic())
            .finish()
    }
}

fn ty_str(ty: &RigidTy) -> String {
    match ty {
        RigidTy::Adt(adt, _) => adt.name(),
        RigidTy::FnDef(fn_def, _) => fn_def.name(),
        RigidTy::Ref(_, ty, mutability) => {
            let ref_ = if matches!(mutability, Mutability::Mut) {
                "&mut "
            } else {
                "&"
            };
            match ty.kind() {
                TyKind::RigidTy(ty) => format!("{ref_}{}", ty_str(&ty)),
                ty => format!("{ref_}{ty:?}"),
            }
        }
        RigidTy::Tuple(v) if v.is_empty() => "()".to_string(),
        ty => format!("{ty:?}"),
    }
}

pub struct Place2 {
    pub place: Place,
    pub span: Span,
}

impl Place2 {
    #[expect(unused)]
    pub fn ty(&self, body: &Body) -> String {
        let span = self.span;
        if let Ok(ty) = self.place.ty(body.locals()) {
            if let TyKind::RigidTy(ty) = ty.kind() {
                format!("{:?}", Type { ty, span })
            } else {
                format!("ty: {ty:?} {span:?}")
            }
        } else {
            format!("Unknown Type; {span:?}")
        }
    }
}

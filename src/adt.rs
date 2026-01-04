use crate::utils::{FxHashMap, FxIndexSet, ThinVec};
use derive_more::Debug;
use rustc_middle::ty::TyCtxt;
use rustc_public::{
    CrateDef,
    mir::ProjectionElem,
    rustc_internal::internal,
    ty::{AdtDef, AdtKind, GenericArgs},
};
use std::sync::Arc;

/// Monomorphized adt.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Adt {
    pub def: AdtDef,
    pub args: GenericArgs,
    pub variant_fields: Arc<Vec<VaraintField>>,
}

impl Adt {
    pub fn new(def: AdtDef, args: GenericArgs) -> Adt {
        Adt {
            def,
            args,
            variant_fields: Arc::new(new_variant_fields(def)),
        }
    }

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

    /// Returns Some iff the adt is struct or union.
    pub fn num_fields(&self) -> Option<usize> {
        for vfield in &*self.variant_fields {
            if !vfield.idx.is_field() {
                return None;
            }
        }
        Some(self.variant_fields.len())
    }
}

fn new_variant_fields(def: AdtDef) -> Vec<VaraintField> {
    let mut variant_fields = Vec::new();
    for (variant_idx, variant) in def.variants_iter().enumerate() {
        let fields = variant.fields();
        match def.kind() {
            AdtKind::Enum => {
                // Enum variant is always pushed even if it carries fields.
                variant_fields.push(VaraintField {
                    idx: VaraintFieldIdx::new_variant(variant_idx),
                    name: variant.name().into(),
                });
                for (field_idx, field) in fields.into_iter().enumerate() {
                    variant_fields.push(VaraintField {
                        idx: VaraintFieldIdx::new_variant_field(variant_idx, field_idx),
                        name: field.name.into(),
                    });
                }
            }
            AdtKind::Struct | AdtKind::Union => {
                if variant_idx != 0 {
                    panic!(
                        "{def:?} is a struct with multiple variants: {:#?}",
                        def.variants_iter().collect::<Vec<_>>()
                    );
                }
                if fields.is_empty() {
                    variant_fields.push(VaraintField {
                        idx: VaraintFieldIdx::unit_struct(),
                        name: Box::default(),
                    });
                } else {
                    for (idx, field) in fields.into_iter().enumerate() {
                        variant_fields.push(VaraintField {
                            idx: VaraintFieldIdx::new_field(idx),
                            name: field.name.into(),
                        });
                    }
                }
            }
        }
    }
    variant_fields
}

/// Access to locals including retern place, argument places,
/// and inner function places.
#[derive(Debug, Default)]
pub struct LocalsAccess {
    /// Local indices. See [`Body::locals`].
    ///
    /// 0 means retern place, 1..=arg_count means argument places,
    /// the rest means inner function places.
    ///
    /// [`Body::locals`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/mir/struct.Body.html#structfield.locals
    pub locals: ThinVec<usize>,
    pub access: FxIndexSet<AdtAccess>,
}

impl LocalsAccess {
    /// Sort and deduplicate indices.
    pub fn deduplicate_indices(&mut self) {
        self.locals.sort_unstable();
        self.locals.dedup();
        self.locals.shrink_to_fit();
    }

    /// Returns true if the local index refers to any argument.
    pub fn is_argument(&self, arg_count: usize) -> bool {
        self.locals
            .iter()
            .any(|&idx| idx < arg_count + 1 && idx != 0)
    }
}

/// Reference to rederence to the adt or its field.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AdtAccess {
    Ref,
    MutRef,
    Deref,
    Plain,
    RefVariantField(VaraintFieldIdx),
    MutRefVariantField(VaraintFieldIdx),
    DerefVariantField(VaraintFieldIdx),
    #[debug("Unknown({:?})", _0)]
    Unknown(Box<[ProjectionElem]>),
}

/// A variant or field for an adt. The representation is pretty flatten for JSON and JS.
///
/// Option types combinations:
/// * `{ variant: None, field: Some }` refers to a struct or union field, like `A { a: Type1, b: Type2 }`.
/// * `{ variant: Some, field: None }` refers to an enum variant, like `A::Ctor`.
/// * `{ variant: Some, field: Some }` refers to a field in an enum variant, like `A::S { c: Type }`;
///   there must an enum variant before this case is pushed into the Vec.
///
/// u32 means the position:
/// * `{ variant: None, field: Some(0) }` means the first field in a struct or union.
/// * `{ variant: Some(0), field: None }` means the first variant in an enum.
/// * `{ variant: Some(1), field: Some(0) }` means the first field in the second enum variant.
///
/// `{ variant: None, field: None }` means the struct is unit (no fields), like `struct S`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[debug("VariantIdx({:?})-FieldIdx({:?})", variant, field)]
pub struct VaraintFieldIdx {
    pub variant: Option<u32>,
    pub field: Option<u32>,
}

impl VaraintFieldIdx {
    pub fn new_field(field_idx: usize) -> Self {
        Self {
            variant: None,
            field: Some(field_idx as u32),
        }
    }

    pub fn new_variant(variant_idx: usize) -> Self {
        Self {
            variant: Some(variant_idx as u32),
            field: None,
        }
    }

    pub fn new_variant_field(variant_idx: usize, field_idx: usize) -> Self {
        Self {
            variant: Some(variant_idx as u32),
            field: Some(field_idx as u32),
        }
    }

    pub fn unit_struct() -> Self {
        Self {
            variant: None,
            field: None,
        }
    }

    pub fn is_field(&self) -> bool {
        self.variant.is_none() & self.field.is_some()
    }

    pub fn field_idx(&self) -> Option<usize> {
        if self.variant.is_none()
            && let Some(idx) = self.field
        {
            Some(idx as usize)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VaraintField {
    pub idx: VaraintFieldIdx,
    pub name: Box<str>,
}

pub type CacheAdt = FxHashMap<AdtDef, Adt>;

/// Retrieve Adt via def. The function is faster because it skips
/// construction of VaraintField if found.
pub fn new_adt(def: AdtDef, args: GenericArgs, cache: &mut CacheAdt) -> Adt {
    let mut adt = cache
        .entry(def)
        .or_insert_with(|| Adt::new(def, args.clone()))
        .clone();
    adt.args = args;
    adt
}

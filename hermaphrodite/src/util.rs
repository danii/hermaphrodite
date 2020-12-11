use std::{marker::PhantomData, mem::transmute, raw::TraitObject};

pub macro generic_trait($traitt:expr) {
	GenericTraitObject {
		trait_object: unsafe {transmute($traitt)},
		phantom: PhantomData
	}
}

pub macro generic_trait_downcast($traitt:expr) {
	transmute($traitt.trait_object)
}

#[derive(Copy, Clone)]
pub struct GenericTraitObject<'p> {
	pub trait_object: TraitObject,
	pub phantom: PhantomData<&'p ()>
}

unsafe impl<'p> Send for GenericTraitObject<'p> {}

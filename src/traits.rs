// Traits

// Provides Internal Pointer type(for wrapper objects)
pub trait InternalProvider<InternalType>
{
	fn get(&self) -> InternalType;
}
// Provides Reference to Parent object
pub trait HasParent
{
	type ParentRefType;
	fn parent(&self) -> Self::ParentRefType;
}

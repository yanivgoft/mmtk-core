pub trait AbstractClass<Base>: Sized + 'static {
    type This: AbstractClass<Base>;

    fn common(this: &Self::This) -> &Base;
    fn common_mut(this: &mut Self::This) -> &mut Base;
}
pub trait CompleteClass<Base>: AbstractClass<Base, This = Self> {
    fn common(&self) -> &Base { <Self::This as AbstractClass<Base>>::common(self) }
    fn common_mut(&mut self) -> &mut Base { <Self::This as AbstractClass<Base>>::common_mut(self) }
}
impl <B, T: AbstractClass<B, This = T>> CompleteClass<B> for T { }

pub trait AbstractMutableClass<Base>: AbstractClass<Base> {
    // UNSAFE: This get's a mutable reference from self
    // (i.e. make sure their are no concurrent accesses through self when calling this)_
    unsafe fn unsafe_common_mut(this: &Self::This) -> &mut Base;
}
pub trait CompleteMutableClass<Base>: CompleteClass<Base, This = Self> + AbstractMutableClass<Base> {
    // UNSAFE: This get's a mutable reference from self
    // (i.e. make sure their are no concurrent accesses through self when calling this)_
    unsafe fn unsafe_common_mut(&self) -> &mut Base  { <Self::This as AbstractMutableClass<Base>>::unsafe_common_mut(self) }
}
impl <B, T: AbstractMutableClass<B, This = T>> CompleteMutableClass<B> for T { }
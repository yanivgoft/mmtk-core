pub trait AbstractClass: Sized + 'static {
    type This: CompleteClass;
}
impl<T: CompleteClass> AbstractClass for T {
    type This = T;
}

pub trait CompleteClass: AbstractClass<This = Self> {
    // The purposes of these functions (as opposed to the '_impl' ones) is so you can disambiguate
    // calls more easily, compare
    //      self.common::<TargetType>()
    // to:
    //      <Self as DerivedClass<TargetType>>::common_impl(self)
    fn common<T>(&self)->&T where Self: DerivedClass<T> {
        self.common_impl()
    }
    fn common_mut<T>(&mut self)->&mut T where Self: DerivedClass<T> {
        self.common_mut_impl()
    }
    unsafe fn unsafe_common_mut<T>(&self)->&mut T where Self: MutableDerivedClass<T> {
        self.unsafe_common_mut_impl()
    }
}
pub trait DerivedClass<Base>: CompleteClass {
    fn common_impl(&self) -> &Base;
    fn common_mut_impl(&mut self) -> &mut Base;
}

pub trait MutableDerivedClass<Base>: DerivedClass<Base> {
    // UNSAFE: This get's a mutable reference from self
    // (i.e. make sure their are no concurrent accesses through self when calling this)_
    unsafe fn unsafe_common_mut_impl(&self) -> &mut Base;
}
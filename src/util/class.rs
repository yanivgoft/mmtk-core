// A piece in a class-hierarchy, but not the most-derived class existing at runtime
// (i.e. your 'runtime' type)
pub trait AbstractClass: Sized + 'static {
    // The run-time type of instances of Self
    // i.e. the 'most derived' class that contains Self, and is not being used as a base-class
    type This: CompleteClass;
}
// A class that is not being used as a base-class (so that This = Self)
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
// Auto-impl AbstractClass, you only need to go 'impl CompleteClass for T {}'
impl<T: CompleteClass> AbstractClass for T {
    type This = T;
}

// Implement this to tell Rust how to get a reference to your 'Base' class from \
// your (indirectly)-DerivedClass 'Self'
// Note: A type can implement DerivedClass<Base> for multiple 'Base's
pub trait DerivedClass<Base>: CompleteClass {
    fn common_impl(&self) -> &Base;
    fn common_mut_impl(&mut self) -> &mut Base;
}

// For type's that allow Mutating the Base even if you only have an immutable reference
// to your Derived class (Self)
pub trait MutableDerivedClass<Base>: DerivedClass<Base> {
    // UNSAFE: This get's a mutable reference from self
    // (i.e. make sure their are no concurrent accesses through self when calling this)_
    unsafe fn unsafe_common_mut_impl(&self) -> &mut Base;
}

// Note: The above two traits (DerivedClass and MutableDerivedClass) require CompleteClass,
// since allowing types to 'override' 'inherited' 'common_impl' function's isn't really useful
// and will severally complicate things.
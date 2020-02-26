#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MMTKHandle<T>(*mut T);

impl<T> MMTKHandle<T> {
    pub fn new(v: T) -> Self {
        MMTKHandle(Box::into_raw(Box::new(v)))
    }

    pub unsafe fn as_mut(self) -> &'static mut T {
        &mut *self.0
    }

    pub fn as_ref(self) -> &'static T {
        unsafe { &*self.0 }
    }
}

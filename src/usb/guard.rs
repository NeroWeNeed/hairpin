use std::marker::PhantomData;

#[derive(Debug)]
pub struct Guard<'a, T> {
    value: std::sync::Arc<tokio::sync::RwLock<T>>,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a, T> Guard<'a, T> {
    pub async fn get(&'a self) -> tokio::sync::RwLockReadGuard<'a, T> {
        self.value.read().await
    }
}
unsafe impl<'a, T> Send for Guard<'a, T> {}
unsafe impl<'a, T> Sync for Guard<'a, T> {}
impl<'a, T> From<T> for Guard<'a, T> {
    fn from(value: T) -> Self {
        Self {
            value: std::sync::Arc::new(tokio::sync::RwLock::new(value)),
            _lifetime: PhantomData::default(),
        }
    }
}
impl<'a, T> Clone for Guard<'a, T> {
    fn clone(&self) -> Self {
        Guard {
            value: self.value.clone(),
            _lifetime: PhantomData::default(),
        }
    }
}

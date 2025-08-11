pub struct WhileNext<T>(Option<T>);

impl<T> WhileNext<T> {
    #[inline(always)]
    pub fn stop() -> Self {
        Self(None)
    }

    pub fn continue_with(value: T) -> Self {
        Self(Some(value))
    }

    #[inline(always)]
    pub fn is_stop(&self) -> bool {
        self.0.is_none()
    }

    #[inline(always)]
    pub fn is_continue(&self) -> bool {
        self.0.is_some()
    }

    #[inline(always)]
    pub fn into_continue(self) -> Option<T> {
        self.0
    }

    #[inline(always)]
    pub fn map<O>(self, map: impl Fn(T) -> O) -> WhileNext<O> {
        WhileNext(self.0.map(map))
    }

    #[inline(always)]
    pub fn filter(self, filter: impl Fn(&T) -> bool) -> Option<Self> {
        match &self.0 {
            Some(x) => match filter(x) {
                true => Some(self),
                false => None,
            },
            None => Some(self),
        }
    }
}

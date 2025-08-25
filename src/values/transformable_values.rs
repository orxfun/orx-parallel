use crate::values::{Values, runner_results::Fallible};

pub trait TransformableValues: Values {
    fn map<M, O>(
        self,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(Self::Item) -> O + Clone;

    fn filter<F>(
        self,
        filter: F,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>
    where
        F: Fn(&Self::Item) -> bool + Clone;

    fn flat_map<Fm, Vo>(
        self,
        flat_map: Fm,
    ) -> impl TransformableValues<Item = Vo::Item, Fallibility = Self::Fallibility>
    where
        Vo: IntoIterator,
        Fm: Fn(Self::Item) -> Vo + Clone;

    fn filter_map<Fm, O>(
        self,
        filter_map: Fm,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        Fm: Fn(Self::Item) -> Option<O>;

    fn whilst(
        self,
        whilst: impl Fn(&Self::Item) -> bool,
    ) -> impl TransformableValues<Item = Self::Item, Fallibility = Self::Fallibility>;

    fn map_while_ok<Mr, O, E>(
        self,
        map_res: Mr,
    ) -> impl Values<Item = O, Fallibility = Fallible<E>>
    where
        Mr: Fn(Self::Item) -> Result<O, E>,
        E: Send;

    fn u_map<U, M, O>(
        self,
        u: &mut U,
        map: M,
    ) -> impl TransformableValues<Item = O, Fallibility = Self::Fallibility>
    where
        M: Fn(&mut U, Self::Item) -> O;
}

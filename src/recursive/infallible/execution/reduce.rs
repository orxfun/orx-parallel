use crate::Par;

pub fn reduce<P, I, E, F>(par: P, extend: E, f: F) -> Option<P::Item>
where
    P: Par,
    I: IntoIterator<Item = P::Item>,
    E: Fn(&P::Item) -> I + Send + Sync,
    F: Fn(P::Item, P::Item) -> P::Item,
{
    todo!()
}

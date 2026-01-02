use std::string::String;

pub(super) fn make_u_map<I, O>(
    map: impl Fn(I) -> O + Clone,
) -> impl Fn(&mut String, I) -> O + Clone {
    move |u: &mut String, x: I| {
        let u = u.as_mut_str();
        u.get_mut(0..2).unwrap().make_ascii_uppercase();
        map(x)
    }
}

pub(super) fn make_u_filter<I>(
    filter: &impl Fn(&I) -> bool,
) -> impl Fn(&mut String, &I) -> bool + Clone {
    |u: &mut String, x: &I| {
        let u = u.as_mut_str();
        u.get_mut(0..2).unwrap().make_ascii_uppercase();
        filter(x)
    }
}

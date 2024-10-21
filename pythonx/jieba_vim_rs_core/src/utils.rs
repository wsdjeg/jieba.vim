pub fn stack_merge<'a, T, U, F, A>(
    elements: Vec<T>,
    args: &A,
    mut rule_func: F,
) -> Vec<U>
where
    F: FnMut(Option<U>, T, &A) -> Vec<U>,
    A: 'a,
{
    let mut stack: Vec<U> = vec![];
    for e in elements {
        let mut merged = rule_func(stack.pop(), e, args);
        stack.append(&mut merged);
    }
    stack
}

pub fn chain_into_vec<T, I, J>(i: I, j: J) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    J: IntoIterator<Item = T>,
{
    i.into_iter().chain(j.into_iter()).collect()
}

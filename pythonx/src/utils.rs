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

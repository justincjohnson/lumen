use super::*;

#[test]
fn with_small_integer_right_returns_true() {
    is_greater_than(|_, process| 0.into_process(&process), true)
}

#[test]
fn with_big_integer_right_returns_true() {
    is_greater_than(
        |_, process| (crate::integer::small::MAX + 1).into_process(&process),
        true,
    )
}

#[test]
fn with_float_right_returns_true() {
    is_greater_than(|_, process| 0.0.into_process(&process), true)
}

#[test]
fn with_greater_atom_returns_true() {
    is_greater_than(|_, _| Term::str_to_atom("keft", DoNotCare).unwrap(), true);
}

#[test]
fn with_same_atom_returns_false() {
    is_greater_than(|left, _| left, false);
}

#[test]
fn with_same_atom_value_returns_false() {
    is_greater_than(|_, _| Term::str_to_atom("left", DoNotCare).unwrap(), false);
}

#[test]
fn with_greater_atom_returns_false() {
    is_greater_than(|_, _| Term::str_to_atom("meft", DoNotCare).unwrap(), false);
}

#[test]
fn with_local_reference_right_returns_false() {
    is_greater_than(|_, process| Term::next_local_reference(process), false);
}

#[test]
fn with_local_pid_right_returns_false() {
    is_greater_than(|_, _| Term::local_pid(0, 1).unwrap(), false);
}

#[test]
fn with_external_pid_right_returns_false() {
    is_greater_than(
        |_, process| Term::external_pid(1, 2, 3, &process).unwrap(),
        false,
    );
}

#[test]
fn with_tuple_right_returns_false() {
    is_greater_than(|_, process| Term::slice_to_tuple(&[], &process), false);
}

#[test]
fn with_map_right_returns_false() {
    is_greater_than(|_, process| Term::slice_to_map(&[], &process), false);
}

#[test]
fn with_empty_list_right_returns_false() {
    is_greater_than(|_, _| Term::EMPTY_LIST, false);
}

#[test]
fn with_list_right_returns_false() {
    is_greater_than(
        |_, process| Term::cons(0.into_process(&process), 1.into_process(&process), &process),
        false,
    );
}

#[test]
fn with_heap_binary_right_returns_false() {
    is_greater_than(|_, process| Term::slice_to_binary(&[], &process), false);
}

#[test]
fn with_subbinary_right_returns_false() {
    is_greater_than(|_, process| bitstring!(1 :: 1, &process), false);
}

fn is_greater_than<R>(right: R, expected: bool)
where
    R: FnOnce(Term, &Process) -> Term,
{
    super::is_greater_than(
        |_| Term::str_to_atom("left", DoNotCare).unwrap(),
        right,
        expected,
    );
}

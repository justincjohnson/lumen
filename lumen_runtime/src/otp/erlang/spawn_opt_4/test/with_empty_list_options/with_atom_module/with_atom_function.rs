use super::*;

mod with_empty_list_arguments;
mod with_non_empty_proper_list_arguments;

#[test]
fn without_proper_list_arguments_errors_badarg() {
    run!(
        |arc_process| {
            (
                Just(arc_process.clone()),
                strategy::term::atom(),
                strategy::term::atom(),
                strategy::term::is_not_list(arc_process.clone()),
            )
        },
        |(arc_process, module, function, arguments)| {
            prop_assert_badarg!(
                native(&arc_process, module, function, arguments, OPTIONS),
                format!("arguments ({}) must be a proper list", arguments)
            );

            Ok(())
        },
    );
}

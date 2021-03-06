use std::sync::Arc;

use liblumen_alloc::Process;

use crate::runtime::process::spawn::options::Options;
use crate::runtime::scheduler::{self, Spawned};
use crate::{erlang, runtime};

use super::loop_0;

pub fn default() -> Arc<Process> {
    child(&init())
}

pub fn init() -> Arc<Process> {
    runtime::test::once(&[
        erlang::apply_3::function_symbol(),
        erlang::exit_1::function_symbol(),
        erlang::number_or_badarith_1::function_symbol(),
        erlang::self_0::function_symbol(),
        super::anonymous_0::function_symbol(),
        super::anonymous_1::function_symbol(),
        super::init::start_0::function_symbol(),
    ]);

    // During test allow multiple unregistered init processes because in tests, the `Scheduler`s
    // keep getting `Drop`ed as threads end.
    scheduler::current()
        .spawn_init(
            // init process being the parent process needs space for the arguments when spawning
            // child processes.  These will not be GC'd, so it can be a lot of space if proptest
            // needs to generate a lot of processes.
            16_000,
        )
        .unwrap()
}

pub fn child(parent_process: &Process) -> Arc<Process> {
    let mut options: Options = Default::default();
    options.min_heap_size = Some(16_000);
    let module = loop_0::module();
    let function = loop_0::function();
    let arguments = &[];
    let native = loop_0::NATIVE;

    let Spawned {
        arc_process: child_arc_process,
        connection,
    } = runtime::process::spawn::native(
        Some(parent_process),
        options,
        module,
        function,
        arguments,
        native,
    )
    .map(|spawned| spawned.schedule_with_parent(parent_process))
    .unwrap();
    assert!(!connection.linked);
    assert!(connection.monitor_reference.is_none());

    child_arc_process
}

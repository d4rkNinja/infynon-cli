mod inner {
    #![allow(clippy::too_many_arguments)]
    include!("commands/dispatch_soul.rs");
    include!("commands/workspace.rs");
    include!("commands/task.rs");
    include!("commands/helpers.rs");
    include!("commands/launch.rs");
    include!("commands/project_agent.rs");
    include!("commands/agent.rs");
}

pub use inner::{
    ensure_first_run_identity_prompt, execute_coding, execute_ninja, execute_soul, execute_task,
    execute_workspace,
};

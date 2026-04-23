use crate::api::commands::{ai_cmd, attach, flow, import, node, validate};
use crate::cli::args::{
    AiAction, ApiCommands, AssertionAction, EnvAction, FlowAction, NodeAction, PromptAction,
};
use crate::tui::logger::Logger;

pub fn execute_api_command(action: ApiCommands) {
    if let Err(message) = crate::cli::validate::validate_api_command(&action) {
        Logger::error(&message);
        return;
    }
    match action {
        ApiCommands::Tui { flow_id } => super::tui::run_api_tui(flow_id.as_deref()),
        ApiCommands::Node { action } => handle_node_action(action),
        ApiCommands::Flow { action } => handle_flow_action(action),
        ApiCommands::Validate => validate::cmd_validate(),
        ApiCommands::Attach {
            from,
            to,
            carry,
            condition,
            ai,
        } => attach::cmd_attach(&from, &to, &carry, condition.as_deref(), ai),
        ApiCommands::Detach { from, to } => attach::cmd_detach(&from, &to),
        ApiCommands::Ai { action } => handle_ai_action(action),
        ApiCommands::Import {
            spec,
            flow,
            base_url,
            prefix,
            dry_run,
        } => import::cmd_import(
            &spec,
            flow.as_deref(),
            base_url.as_deref(),
            prefix.as_deref(),
            dry_run,
        ),
        ApiCommands::Env { action } => match action {
            EnvAction::List => crate::api::commands::env::cmd_env_list(),
            EnvAction::Set { key, value } => crate::api::commands::env::cmd_env_set(&key, &value),
            EnvAction::Delete { key } => crate::api::commands::env::cmd_env_delete(&key),
            EnvAction::Get { key, reveal } => crate::api::commands::env::cmd_env_get(&key, reveal),
        },
    }
}

fn handle_node_action(action: NodeAction) {
    match action {
        NodeAction::Create { ai } => node::cmd_node_create(ai.as_deref()),
        NodeAction::Get { id } => node::cmd_node_get(&id),
        NodeAction::List => node::cmd_node_list(),
        NodeAction::Remove { id } => node::cmd_node_remove(&id),
        NodeAction::Clone { id, new_id } => node::cmd_node_clone(&id, &new_id),
        NodeAction::Run {
            id,
            base_url,
            set,
            prompt,
        } => {
            let url = match base_url.or_else(crate::api::commands::env::env_base_url) {
                Some(url) => url,
                None => {
                    return Logger::error(
                        "BASE_URL is not set. Add it to .infynon/.env or pass --base-url <url>",
                    )
                }
            };
            node::cmd_node_run(&id, &url, &set, prompt);
        }
        NodeAction::Export {
            id,
            format,
            base_url,
        } => node::cmd_node_export(&id, &format, base_url.as_deref()),
        NodeAction::Assertion { node_id, action } => match action {
            AssertionAction::List => node::cmd_node_assertion_list(&node_id),
            AssertionAction::Enable { index } => node::cmd_node_assertion_enable(&node_id, index),
            AssertionAction::Disable { index } => node::cmd_node_assertion_disable(&node_id, index),
            AssertionAction::Toggle { index } => node::cmd_node_assertion_toggle(&node_id, index),
            AssertionAction::Add { check, on_fail } => {
                node::cmd_node_assertion_add(&node_id, &check, &on_fail)
            }
            AssertionAction::Remove { index } => node::cmd_node_assertion_remove(&node_id, index),
        },
        NodeAction::Prompt { node_id, action } => match action {
            PromptAction::List => node::cmd_node_prompt_list(&node_id),
            PromptAction::Add {
                var,
                label,
                secret,
                default,
                prompt_type,
                options,
            } => node::cmd_node_prompt_add(
                &node_id,
                &var,
                &label,
                secret,
                default,
                &prompt_type,
                options,
            ),
            PromptAction::Remove { index } => node::cmd_node_prompt_remove(&node_id, index),
        },
    }
}

fn handle_flow_action(action: FlowAction) {
    match action {
        FlowAction::Create { name, ai } => flow::cmd_flow_create(&name, ai.as_deref()),
        FlowAction::List => flow::cmd_flow_list(),
        FlowAction::Show { id } => flow::cmd_flow_show(&id),
        FlowAction::Run {
            id,
            base_url,
            set,
            format,
            output,
            no_input,
        } => std::process::exit(flow::cmd_flow_run(
            &id,
            base_url.as_deref(),
            &set,
            format.as_deref(),
            output.as_deref(),
            no_input,
        )),
        FlowAction::RunAll {
            base_url,
            set,
            format,
            output,
            no_input,
        } => std::process::exit(flow::cmd_flow_run_all(
            base_url.as_deref(),
            &set,
            format.as_deref(),
            output.as_deref(),
            no_input,
        )),
        FlowAction::Remove { id } => flow::cmd_flow_remove(&id),
        FlowAction::Merge {
            flow1,
            flow2,
            join_at,
            name,
        } => flow::cmd_flow_merge(&flow1, &flow2, &join_at, &name),
    }
}

fn handle_ai_action(action: AiAction) {
    match action {
        AiAction::Suggest { after } => ai_cmd::cmd_ai_suggest(&after),
        AiAction::Attach { after, flow } => ai_cmd::cmd_ai_attach(&after, flow.as_deref()),
        AiAction::Complete { flow_id } => ai_cmd::cmd_ai_complete(&flow_id),
        AiAction::Probe { flow_id, base_url } => {
            ai_cmd::cmd_ai_probe(&flow_id, base_url.as_deref())
        }
        AiAction::BuildFlow { nodes, name } => ai_cmd::cmd_ai_build_flow(&nodes, &name),
        AiAction::Explain { flow_id, run } => ai_cmd::cmd_ai_explain(&flow_id, run),
        AiAction::Assert { node_id } => ai_cmd::cmd_ai_assert(&node_id),
        AiAction::Branch { node_id } => ai_cmd::cmd_ai_branch(&node_id),
    }
}

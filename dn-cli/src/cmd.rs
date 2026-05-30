use crate::args;
use crate::context;
use crate::epic;
use crate::file;
use crate::issue;
use crate::knowledge;
use crate::link;
use crate::module;
use crate::persona;
use crate::project;
use crate::task;
use crate::todo;
use crate::user_story;
use crate::workflow;

pub(crate) async fn dispatch(
    command: args::Commands,
    db: &dn_core::db::surreal::DB,
    pretty: bool,
    ignore_case: bool,
) -> anyhow::Result<()> {
    match command {
        args::Commands::Add {
            field_names,
            field_values,
            link_to,
        } => knowledge::handle_add(field_names, field_values, link_to, db, pretty).await,
        args::Commands::Link {
            from_id,
            edge,
            to_id,
        } => link::handle_link(from_id, edge, to_id, db, pretty).await,
        args::Commands::Project { command } => {
            project::handle_project_command(command, db, pretty).await
        }
        args::Commands::Module { command } => {
            module::handle_module_command(command, db, pretty, ignore_case).await
        }
        args::Commands::File { command } => {
            file::handle_file_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Task { command } => {
            task::handle_task_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Todo { command } => {
            todo::handle_todo_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Issue { command } => {
            issue::handle_issue_command(command, db, pretty, ignore_case).await
        }
        args::Commands::UserStory { command } => {
            user_story::handle_user_story_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Epic { command } => {
            epic::handle_epic_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Persona { command } => {
            persona::handle_persona_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Workflow { command } => {
            workflow::handle_workflow_command(command, db, pretty, ignore_case).await
        }
        args::Commands::Context {
            task_id,
            file_id,
            epic_id,
            full,
            general,
            project,
        } => {
            context::handle_context(
                task_id, file_id, epic_id, full, general, project, db, pretty,
            )
            .await
        }
        args::Commands::Rm { context_ids } => knowledge::handle_rm(context_ids, db, pretty).await,
        args::Commands::Purge => context::handle_purge(db, pretty).await,
        args::Commands::Config { .. } => {
            unreachable!("config command handled before db init")
        }
    }
}

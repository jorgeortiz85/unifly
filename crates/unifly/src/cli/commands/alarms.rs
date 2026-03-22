//! Alarm command handlers.

use tabled::Tabled;
use unifly_api::{Alarm, Command as CoreCommand, Controller, EntityId};

use crate::cli::args::{AlarmsArgs, AlarmsCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct AlarmRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Time")]
    time: String,
    #[tabled(rename = "Severity")]
    severity: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Message")]
    message: String,
    #[tabled(rename = "Archived")]
    archived: String,
}

fn alarm_row(a: &Alarm, p: &output::Painter) -> AlarmRow {
    AlarmRow {
        id: p.id(&a.id.to_string()),
        time: p.muted(&a.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
        severity: p.health(&format!("{:?}", a.severity)),
        category: p.muted(&format!("{:?}", a.category)),
        message: a.message.clone(),
        archived: p.enabled(a.archived),
    }
}

// ── Handler ─────────────────────────────────────────────────────────

pub async fn handle(
    controller: &Controller,
    args: AlarmsArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let p = output::Painter::new(global);

    match args.command {
        AlarmsCommand::List { unarchived, limit } => {
            let mut alarms = controller.list_alarms().await?;
            if unarchived {
                alarms.retain(|a| !a.archived);
            }
            alarms.truncate(usize::try_from(limit).unwrap_or(usize::MAX));
            let out = output::render_list(
                &global.output,
                &alarms,
                |a| alarm_row(a, &p),
                |a| a.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        AlarmsCommand::Archive { id } => {
            let eid = EntityId::from(id);
            controller
                .execute(CoreCommand::ArchiveAlarm { id: eid })
                .await?;
            if !global.quiet {
                eprintln!("Alarm archived");
            }
            Ok(())
        }

        AlarmsCommand::ArchiveAll => {
            if !util::confirm("Archive all alarms?", global.yes)? {
                return Ok(());
            }
            controller.execute(CoreCommand::ArchiveAllAlarms).await?;
            if !global.quiet {
                eprintln!("All alarms archived");
            }
            Ok(())
        }
    }
}

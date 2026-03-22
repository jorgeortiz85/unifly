//! Admin command handlers.

use tabled::Tabled;
use unifly_api::{Admin, Command as CoreCommand, Controller, EntityId};

use crate::cli::args::{AdminArgs, AdminCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct AdminRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Email")]
    email: String,
    #[tabled(rename = "Role")]
    role: String,
    #[tabled(rename = "Super")]
    is_super: String,
}

fn admin_row(a: &Admin, p: &output::Painter) -> AdminRow {
    AdminRow {
        id: p.id(&a.id.to_string()),
        name: p.name(&a.name),
        email: p.muted(&a.email.clone().unwrap_or_default()),
        role: p.muted(&a.role),
        is_super: p.enabled(a.is_super),
    }
}

// ── Handler ─────────────────────────────────────────────────────────

pub async fn handle(
    controller: &Controller,
    args: AdminArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let p = output::Painter::new(global);

    match args.command {
        AdminCommand::List => {
            let admins = controller.list_admins().await?;
            let out = output::render_list(
                &global.output,
                &admins,
                |a| admin_row(a, &p),
                |a| a.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        AdminCommand::Invite { name, email, role } => {
            controller
                .execute(CoreCommand::InviteAdmin { name, email, role })
                .await?;
            if !global.quiet {
                eprintln!("Admin invitation sent");
            }
            Ok(())
        }

        AdminCommand::Revoke { admin } => {
            let id = EntityId::from(admin.clone());
            if !util::confirm(&format!("Revoke admin access for {admin}?"), global.yes)? {
                return Ok(());
            }
            controller.execute(CoreCommand::RevokeAdmin { id }).await?;
            if !global.quiet {
                eprintln!("Admin access revoked");
            }
            Ok(())
        }

        AdminCommand::Update { admin, role } => {
            let id = EntityId::from(admin);
            controller
                .execute(CoreCommand::UpdateAdmin {
                    id,
                    role: Some(role),
                })
                .await?;
            if !global.quiet {
                eprintln!("Admin role updated");
            }
            Ok(())
        }
    }
}

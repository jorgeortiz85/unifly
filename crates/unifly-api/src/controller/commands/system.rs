use crate::command::{Command, CommandResult};
use crate::core_error::CoreError;
use crate::model::Voucher;

use super::{CommandContext, require_integration, require_legacy, require_uuid};

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub(super) async fn route(ctx: &CommandContext, cmd: Command) -> Result<CommandResult, CoreError> {
    let integration = ctx.integration.as_ref();
    let legacy = ctx.legacy.as_ref();
    let site_id = ctx.site_id;

    match cmd {
        Command::ArchiveAlarm { id } => {
            let legacy = require_legacy(legacy)?;
            legacy.archive_alarm(&id.to_string()).await?;
            Ok(CommandResult::Ok)
        }
        Command::ArchiveAllAlarms => {
            let legacy = require_legacy(legacy)?;
            legacy.archive_all_alarms().await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateBackup => {
            let legacy = require_legacy(legacy)?;
            legacy.create_backup().await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteBackup { filename } => {
            let legacy = require_legacy(legacy)?;
            legacy.delete_backup(&filename).await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateVouchers(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateVouchers")?;
            #[allow(clippy::as_conversions, clippy::cast_possible_wrap)]
            let body = crate::integration_types::VoucherCreateRequest {
                name: req.name.unwrap_or_else(|| "Voucher".into()),
                count: Some(req.count as i32),
                time_limit_minutes: i64::from(req.time_limit_minutes.unwrap_or(60)),
                authorized_guest_limit: req.authorized_guest_limit.map(i64::from),
                data_usage_limit_m_bytes: req.data_usage_limit_mb.map(|m| m as i64),
                rx_rate_limit_kbps: req.rx_rate_limit_kbps.map(|r| r as i64),
                tx_rate_limit_kbps: req.tx_rate_limit_kbps.map(|r| r as i64),
            };
            let vouchers = ic.create_vouchers(&sid, &body).await?;
            let domain_vouchers: Vec<Voucher> = vouchers.into_iter().map(Voucher::from).collect();
            Ok(CommandResult::Vouchers(domain_vouchers))
        }
        Command::DeleteVoucher { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteVoucher")?;
            let uuid = require_uuid(&id)?;
            ic.delete_voucher(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        Command::PurgeVouchers { filter } => {
            let (ic, sid) = require_integration(integration, site_id, "PurgeVouchers")?;
            ic.purge_vouchers(&sid, &filter).await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateSite { name, description } => {
            let legacy = require_legacy(legacy)?;
            legacy.create_site(&name, &description).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteSite { name } => {
            let legacy = require_legacy(legacy)?;
            legacy.delete_site(&name).await?;
            Ok(CommandResult::Ok)
        }
        Command::InviteAdmin { name, email, role } => {
            let legacy = require_legacy(legacy)?;
            legacy.invite_admin(&name, &email, &role).await?;
            Ok(CommandResult::Ok)
        }
        Command::RevokeAdmin { id } => {
            let legacy = require_legacy(legacy)?;
            legacy.revoke_admin(&id.to_string()).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateAdmin { id, role } => {
            let legacy = require_legacy(legacy)?;
            legacy
                .update_admin(&id.to_string(), role.as_deref())
                .await?;
            Ok(CommandResult::Ok)
        }
        Command::RebootController => {
            let legacy = require_legacy(legacy)?;
            legacy.reboot_controller().await?;
            Ok(CommandResult::Ok)
        }
        Command::PoweroffController => {
            let legacy = require_legacy(legacy)?;
            legacy.poweroff_controller().await?;
            Ok(CommandResult::Ok)
        }
        _ => unreachable!("system::route received non-system command"),
    }
}

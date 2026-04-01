use crate::core_error::CoreError;
use crate::model::{Admin, Alarm, EntityId, HealthSummary, SysInfo, SystemInfo};

use super::Controller;
use super::support::{convert_health_summaries, require_legacy};

impl Controller {
    pub async fn list_backups(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.list_backups().await?)
    }

    pub async fn download_backup(&self, filename: &str) -> Result<Vec<u8>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.download_backup(filename).await?)
    }

    pub async fn get_site_stats(
        &self,
        interval: &str,
        start: Option<i64>,
        end: Option<i64>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_site_stats(interval, start, end, attrs).await?)
    }

    pub async fn get_device_stats(
        &self,
        interval: &str,
        macs: Option<&[String]>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_device_stats(interval, macs, attrs).await?)
    }

    pub async fn get_client_stats(
        &self,
        interval: &str,
        macs: Option<&[String]>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_client_stats(interval, macs, attrs).await?)
    }

    pub async fn get_gateway_stats(
        &self,
        interval: &str,
        start: Option<i64>,
        end: Option<i64>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy
            .get_gateway_stats(interval, start, end, attrs)
            .await?)
    }

    pub async fn get_dpi_stats(
        &self,
        group_by: &str,
        macs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_dpi_stats(group_by, macs).await?)
    }

    pub async fn list_admins(&self) -> Result<Vec<Admin>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_admins().await?;
        Ok(raw
            .into_iter()
            .map(|value| Admin {
                id: value
                    .get("_id")
                    .and_then(|value| value.as_str())
                    .map_or_else(
                        || EntityId::Legacy("unknown".into()),
                        |value| EntityId::Legacy(value.into()),
                    ),
                name: value
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_owned(),
                email: value
                    .get("email")
                    .and_then(|value| value.as_str())
                    .map(String::from),
                role: value
                    .get("role")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_owned(),
                is_super: value
                    .get("is_super")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                last_login: None,
            })
            .collect())
    }

    pub async fn list_users(
        &self,
    ) -> Result<Vec<crate::legacy::models::LegacyUserEntry>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.list_users().await?)
    }

    pub async fn list_alarms(&self) -> Result<Vec<Alarm>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_alarms().await?;
        Ok(raw.into_iter().map(Alarm::from).collect())
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, CoreError> {
        {
            let guard = self.inner.integration_client.lock().await;
            if let Some(ic) = guard.as_ref() {
                let info = ic.get_info().await?;
                let fields = &info.fields;
                return Ok(SystemInfo {
                    controller_name: fields
                        .get("applicationName")
                        .or_else(|| fields.get("name"))
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    version: fields
                        .get("applicationVersion")
                        .or_else(|| fields.get("version"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    build: fields
                        .get("build")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    hostname: fields
                        .get("hostname")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    ip: None,
                    uptime_secs: fields.get("uptime").and_then(serde_json::Value::as_u64),
                    update_available: fields
                        .get("isUpdateAvailable")
                        .or_else(|| fields.get("update_available"))
                        .and_then(serde_json::Value::as_bool),
                });
            }
        }

        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SystemInfo {
            controller_name: raw
                .get("controller_name")
                .or_else(|| raw.get("name"))
                .and_then(|value| value.as_str())
                .map(String::from),
            version: raw
                .get("version")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            build: raw
                .get("build")
                .and_then(|value| value.as_str())
                .map(String::from),
            hostname: raw
                .get("hostname")
                .and_then(|value| value.as_str())
                .map(String::from),
            ip: raw
                .get("ip_addrs")
                .and_then(|value| value.as_array())
                .and_then(|values| values.first())
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse().ok()),
            uptime_secs: raw.get("uptime").and_then(serde_json::Value::as_u64),
            update_available: raw
                .get("update_available")
                .and_then(serde_json::Value::as_bool),
        })
    }

    pub async fn get_site_health(&self) -> Result<Vec<HealthSummary>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_health().await?;
        Ok(convert_health_summaries(raw))
    }

    pub async fn get_sysinfo(&self) -> Result<SysInfo, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SysInfo {
            timezone: raw
                .get("timezone")
                .and_then(|value| value.as_str())
                .map(String::from),
            autobackup: raw.get("autobackup").and_then(serde_json::Value::as_bool),
            hostname: raw
                .get("hostname")
                .and_then(|value| value.as_str())
                .map(String::from),
            ip_addrs: raw
                .get("ip_addrs")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            live_chat: raw
                .get("live_chat")
                .and_then(|value| value.as_str())
                .map(String::from),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            data_retention_days: raw
                .get("data_retention_days")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as u32),
            extra: raw,
        })
    }
}

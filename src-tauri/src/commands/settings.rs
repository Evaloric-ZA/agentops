//! Application settings.

use rusqlite::params;

use crate::db::Database;
use crate::domain::AppSettings;
use crate::error::{AppError, AppResult};

/// Read the settings.
pub fn get(db: &Database) -> AppResult<AppSettings> {
    let mut settings = AppSettings::default();
    let mut stmt = db.conn().prepare("SELECT key, value FROM app_settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (key, value) = row?;
        match key.as_str() {
            "theme" => settings.theme = value,
            "compact_mode" => settings.compact_mode = value == "true",
            "simulator_speed" => {
                if let Ok(speed) = value.parse::<f64>() {
                    settings.simulator_speed = speed;
                }
            }
            "timezone" => settings.timezone = value,
            "notifications_enabled" => settings.notifications_enabled = value == "true",
            _ => {}
        }
    }
    Ok(settings)
}

/// Write the settings, validating each field before it is stored.
pub fn update(db: &Database, settings: &AppSettings) -> AppResult<AppSettings> {
    let theme = match settings.theme.as_str() {
        "light" | "dark" => settings.theme.as_str(),
        other => {
            return Err(AppError::InvalidInput(format!(
                "theme must be 'light' or 'dark', got '{other}'"
            )))
        }
    };
    let timezone = crate::error::validate_text("timezone", &settings.timezone, 64)?;
    if !timezone.is_empty() && timezone != "UTC" && !timezone.contains('/') {
        return Err(AppError::InvalidInput(
            "timezone must be 'UTC' or an IANA name such as 'Africa/Johannesburg'".into(),
        ));
    }
    if !settings.simulator_speed.is_finite() {
        return Err(AppError::InvalidInput(
            "simulator speed must be a finite number".into(),
        ));
    }
    let speed = settings.simulator_speed.clamp(0.25, 4.0);

    let pairs: [(&str, String); 5] = [
        ("theme", theme.to_string()),
        ("compact_mode", settings.compact_mode.to_string()),
        ("simulator_speed", format!("{speed:.2}")),
        ("timezone", timezone),
        (
            "notifications_enabled",
            settings.notifications_enabled.to_string(),
        ),
    ];
    for (key, value) in pairs {
        db.conn().execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
    }
    get(db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed;

    fn prepared() -> Database {
        let db = Database::open_memory().expect("database");
        seed::seed(db.conn(), "2026-01-01T18:00:00Z").expect("seed");
        db
    }

    #[test]
    fn settings_round_trip_through_the_database() {
        let db = prepared();
        let updated = update(
            &db,
            &AppSettings {
                theme: "light".into(),
                compact_mode: true,
                simulator_speed: 2.0,
                timezone: "UTC".into(),
                notifications_enabled: false,
            },
        )
        .expect("update");
        assert_eq!(updated.theme, "light");
        assert!(updated.compact_mode);
        let reread = get(&db).expect("get");
        assert_eq!(reread.theme, "light");
        assert_eq!(reread.simulator_speed, 2.0);
    }

    #[test]
    fn an_unknown_theme_is_rejected() {
        let db = prepared();
        let settings = AppSettings {
            theme: "neon".into(),
            ..AppSettings::default()
        };
        assert!(update(&db, &settings).is_err());
    }

    #[test]
    fn a_malformed_timezone_is_rejected() {
        let db = prepared();
        let settings = AppSettings {
            timezone: "not a zone".into(),
            ..AppSettings::default()
        };
        assert!(update(&db, &settings).is_err());
    }
}

use anyhow::{Context, Result};

const DB_TABLE: &str = "knzhou_hours";
const DB_NAME: &str = if cfg!(debug_assertions) {
    "knzhou.test.db"
} else {
    "knzhou.db"
};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StudySession {
    pub focused: f32,
    pub unfocused: f32,
    pub day: Option<jiff::civil::DateTime>,
}

impl From<crate::cli::HoursLogged> for StudySession {
    fn from(hours: crate::cli::HoursLogged) -> Self {
        let (focused, unfocused) = hours.hours();
        Self {
            focused,
            unfocused,
            day: None,
        }
    }
}

#[derive(Debug)]
pub struct Database {
    conn: rusqlite::Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let mut db_dir = dirs::data_dir().expect("Should have a data directory.");
        db_dir.push("knzhou");
        std::fs::create_dir_all(&db_dir).expect("Should be able to create data directory.");
        let db_path = db_dir.join(DB_NAME);

        let conn = rusqlite::Connection::open(db_path)?;
        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {DB_TABLE} (
                    id INTEGER PRIMARY KEY,
                    focused REAL NOT NULL,
                    unfocused REAL NOT NULL,
                    day TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )"
            ),
            [],
        )
        .with_context(|| "Creating DB table for hours")?;
        Ok(Self { conn })
    }

    pub fn detailed_hours_logged(&self) -> Result<Vec<StudySession>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT focused, unfocused, day FROM {DB_TABLE} ORDER BY day"))?;

        // Create the iterator and return it as a Result
        let query = stmt.query_map([], |row| {
            let focused: f32 = row.get(0)?;
            let unfocused: f32 = row.get(1)?;
            let day = row.get::<_, String>(2)?.parse().ok();
            Ok(StudySession {
                focused,
                unfocused,
                day,
            })
        })?;

        let mut result = Vec::new();
        for session in query {
            result.push(session?);
        }
        Ok(result)
    }

    /// Returns a tuple of the total focused and unfocused hours logged.
    pub fn total_hours_logged(&self) -> Result<(f32, f32)> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT TOTAL(focused), TOTAL(unfocused) FROM {DB_TABLE}"
        ))?;
        let (focused, unfocused) = stmt.query_row([], |r| {
            let focused = r.get::<_, f32>(0)?;
            let unfocused = r.get::<_, f32>(1)?;
            Ok((focused, unfocused))
        })?;
        Ok((focused, unfocused))
    }

    pub fn add_hours(&self, hours: &StudySession) -> Result<()> {
        self.conn.execute(
            &format!("INSERT INTO {DB_TABLE} (focused, unfocused) VALUES (?1, ?2)"),
            [hours.focused, hours.unfocused],
        ).with_context(|| "Adding hours to the database")?;
        Ok(())
    }
}

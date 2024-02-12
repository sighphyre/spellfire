use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use sqlx::{Connection, Row, SqliteConnection};
use tokio;

struct Memory {
    connection: SqliteConnection,
}

struct Character {
    id: i64,
    name: String,
    relationship: i64,
}

enum MemoryError {
    SqlxError(sqlx::Error),
    CharacterError,
}

impl Memory {
    async fn get_character(&mut self) -> Result<Vec<Character>, MemoryError> {
        sqlx::query_as!(Character, "SELECT * FROM known_character")
            .fetch_all(&mut self.connection)
            .await
            .map_err(MemoryError::SqlxError)
    }
}

// #[tokio::main]
async fn build_character(db_path: &str) -> Result<(), sqlx::Error> {
    dotenv().ok();

    println!(
        "Current working path: {:?}",
        std::env::current_dir().unwrap()
    );

    // Connect to the database (the URL can also be provided directly here)
    let mut connection = SqliteConnection::connect(db_path).await?;

    sqlx::migrate!().run(&mut connection).await?;

    // sqlx::query(
    //     "CREATE TABLE IF NOT EXISTS conversation (
    //         character_id INTEGER NOT NULL,
    //         raw_text TEXT NOT NULL,
    //         summary TEXT NOT NULL,
    //         FOREIGN KEY (character_id) REFERENCES known_characters(id)
    //      )",
    // )
    // .execute(&pool)
    // .await?;

    // // Example: Inserting data
    // sqlx::query("INSERT INTO known_characters (name) VALUES (?)")
    //     .bind("Character Name")
    //     .bind("A brief description of the character")
    //     .execute(&pool)
    //     .await?;

    // // Example: Querying data
    // let row = sqlx::query("SELECT name, description FROM characters WHERE id = ?")
    //     .bind(1)
    //     .fetch_one(&pool)
    //     .await?;

    // let name: String = row.get(0);
    // let description: String = row.get(1);

    // println!("Name: {}, Description: {}", name, description);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_build_character() {
        //print out the current environment variables

        // build_character("sqlite::memory:").await.unwrap();
        build_character("sqlite:test.sqlite").await.unwrap();
    }
}

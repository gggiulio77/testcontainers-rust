use sqlx::{postgres::PgPoolOptions, prelude::FromRow, PgPool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    println!("Hello, world!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    println!("Inserting todo");

    let result = add_todo(&pool, "Search job".to_string()).await?;

    println!("Reading todo");

    let todo = get_todo(&pool, result).await?;

    println!("{:?}", result);
    println!("{:?}", todo);

    Ok(())
}

#[derive(Debug, FromRow, PartialEq, Eq)]
struct Todo {
    id: i64,
    description: String,
    done: bool,
}

async fn add_todo(pool: &PgPool, description: String) -> Result<i64, Box<dyn std::error::Error>> {
    let rec: (i64,) = sqlx::query_as(r#"INSERT INTO todos (description) VALUES ($1) RETURNING id"#)
        .bind(description)
        .fetch_one(pool)
        .await?;

    Ok(rec.0)
}

async fn get_todo(pool: &PgPool, id: i64) -> Result<Option<Todo>, Box<dyn std::error::Error>> {
    let result: Option<Todo> =
        sqlx::query_as(r#"SELECT id, description, done FROM todos WHERE id = $1"#)
            .bind(id)
            .fetch_optional(pool)
            .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    #[tokio::test(flavor = "current_thread")]
    async fn test_add_todo() -> Result<(), Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        println!("Creating db container");

        let container = Postgres::default()
            .with_db_name("todos")
            .with_user("testing")
            .with_password("testing")
            .start()
            .await;

        // let container = GenericImage::new("postgres", "latest")
        //     .with_exposed_port(5432)
        //     .with_env_var("POSTGRES_PASSWORD", "testing")
        //     .with_env_var("POSTGRES_USER", "testing")
        //     .with_env_var("POSTGRES_DB", "todos")
        //     .with_wait_for(WaitFor::message_on_stderr(
        //         "database system is ready to accept connections",
        //     ))
        //     // .with_wait_for(WaitFor::Duration {
        //     //     length: Duration::from_secs(10),
        //     // })
        //     .start()
        //     .await;

        let host = container.get_host().await;
        let port = container.get_host_port_ipv4(5432).await;

        println!("Trying to connect to {:?}:{:?}", host, port);

        let connection_string = format!("postgres://testing:testing@{}:{}/todos", host, port);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;

        println!("Running migration");

        sqlx::migrate!("./migrations").run(&pool).await?;

        println!("Migration complete");

        let id = add_todo(&pool, "Search job".to_string()).await?;

        let result = get_todo(&pool, id).await?;

        assert_eq!(
            result,
            Some(Todo {
                id,
                description: "Search job".to_string(),
                done: false
            })
        );

        match result {
            Some(todo) => println!("{:?}", todo),
            None => println!("Something went wrong"),
        };

        Ok(())
    }
}

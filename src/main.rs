use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
mod database;

use database::connection::DB;
use sqlx::types::Uuid;

struct LibrosDb {
    id: Uuid,
    titulo: String,
    descripcion: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct LibrosDBV2 {
    uuid: String,
    titulo: String,
    descripcion: String,
}

#[derive(Serialize)]
struct DefaultResponse {
    status: String,
    libro: Vec<LibrosDBV2>,
    descripcion: String,
}

struct LibroResumen {
    titulo: String,
    descripcion: Option<String>,
}

#[derive(Deserialize)]
struct UuidLibro {
    id: String,
}
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/login", post(login))
        .route("/libros/:id", get(libros))
        .route("/repos", get(repositorios))
        .route("/lista/libros", get(get_libros))
        .route("/libros/one", post(get_libro_uuid))
        .route("/insert", post(insert_libros))
        .route("/update", post(update_libro))
        .route("/delete", post(deleleLibro));

    axum::Server::bind(&"0.0.0.0:7000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct insertlibros {
    titulo: String,
    descripcion: String,
}

#[derive(Serialize)]
struct insertResponse {
    status: String,
    id: String,
    descripcion: String,
}

#[derive(Serialize)]
struct UpdateResponse {
    status: String,
    rows_affected: bool,
    descripcion: String,
}
async fn insert_libros(Json(payload): Json<insertlibros>) -> Json<insertResponse> {
    let pool = DB::connection().await;
    let record = sqlx::query!(
        r#"
    INSERT INTO repolib (titulo, descripcion) values($1, $2) returning id
    "#,
        payload.titulo,
        payload.descripcion
    )
    .fetch_one(&pool)
    .await;

    let id = match record {
        Ok(res) => res.id.to_string(),
        Err(_err) => "".to_string(),
    };

    let response = insertResponse {
        status: "200 OK".to_string(),
        id: id,
        descripcion: "".to_string(),
    };

    Json(response)
}

async fn update_libro(Json(payload): Json<LibrosDBV2>) -> Json<UpdateResponse> {
    let pool = DB::connection().await;
    let Uuid = Uuid::parse_str(&payload.uuid).expect("Invalid parameter");
    let rows_affected = sqlx::query!(
        r#"
   UPDATE repolib
   SET titulo = $1, descripcion = $2 WHERE id = $3
    "#,
        payload.titulo,
        payload.descripcion,
        Uuid
    )
    .execute(&pool)
    .await
    .expect("error al actualizar los datos")
    .rows_affected();

    let response = if rows_affected > 0 {
        UpdateResponse {
            status: "200 OK".to_string(),
            rows_affected: true,
            descripcion: "libro actualizado".to_string(),
        }
    } else {
        UpdateResponse {
            status: "409 conflict".to_string(),
            rows_affected: true,
            descripcion: "error al  actualizado libro".to_string(),
        }
    };

    Json(response)
}

async fn deleleLibro(Json(payload): Json<UuidLibro>) -> Json<UpdateResponse> {
    let pool = DB::connection().await;
    let uuid = Uuid::parse_str(&payload.id).expect("error al transformar uuid");
    let row_delete = sqlx::query!(
        r#"
        
        DELETE FROM repolib WHERE id = $1
        
        "#,
        uuid
    )
    .execute(&pool)
    .await
    .expect("error al eliminar uuid")
    .rows_affected();

    let response = if row_delete > 0 {
        UpdateResponse {
            status: "200 OK".to_string(),
            rows_affected: true,
            descripcion: "libro eliminado".to_string(),
        }
    } else {
        UpdateResponse {
            status: "409 conflict".to_string(),
            rows_affected: true,
            descripcion: "error al eliminar libro".to_string(),
        }
    };

    Json(response)
}

async fn get_libro_uuid(Json(payload): Json<UuidLibro>) -> Json<DefaultResponse> {
    let pool = DB::connection().await;
    let id = Uuid::parse_str(&payload.id).expect("error al transformar uuid");
    let sql = sqlx::query_as!(
        LibroResumen,
        r#"SELECT  titulo , descripcion from repolib WHERE id = $1"#,
        id
    )
    .fetch_one(&pool)
    .await;

    let response = match sql {
        Ok(res) => DefaultResponse {
            status: "200 OK".to_string(),
            libro: vec![],
            descripcion: "registro obtenido".to_string(),
        },
        Err(_err) => DefaultResponse {
            status: "404 Not Found".to_string(),
            libro: vec![],
            descripcion: "libro no encontrado".to_string(),
        },
    };
    Json(response)
}

async fn get_libros() -> Json<DefaultResponse> {
    let pool = DB::connection().await;
    let libros = sqlx::query_as!(LibrosDb, 
        r#"SELECT * FROM repolib"#)
        .fetch_all(&pool)
        .await;

    let response = match libros {
        Ok(res) => DefaultResponse {
            status: "200 OK".to_string(),
            libro: res
                .into_iter()
                .map(|x| LibrosDBV2 {
                    uuid: x.id.to_string(),
                    titulo: x.titulo,
                    descripcion: option_to_string(x.descripcion),
                })
                .collect(),
            descripcion: "Registros obtenidos".to_string(),
        },
        Err(_err) => DefaultResponse {
            status: "409 conflict".to_string(),
            libro: vec![],
            descripcion: "error al obtener registros".to_string(),
        },
    };

    Json(response)
    //las funciones asincronas devuelven un future
    //await extraer la informacion que contenia esa funcion asincona
}
fn option_to_string(descripcion: Option<String>) -> String {
    let resultado = match descripcion {
        Some(desc) => desc,
        None => "".to_string(),
    };

    resultado
}

// https://0.0.0.0libros/:id
#[derive(Serialize)]
struct ResponseLibros {
    id: String,
    status: String,
    descripcion: String,
}

async fn libros(Path(id): Path<String>) -> Json<ResponseLibros> {
    let libro_id = id;
    let sql = format!("SELECT * FROM libros where id = {libro_id}");

    let response = ResponseLibros {
        id: libro_id,
        status: "OK".to_string(),
        descripcion: "Libro".to_string(),
    };

    Json(response)
}

#[derive(Serialize, Deserialize)]
struct Search {
    language: String,
    type_lan: String,
}

async fn repositorios(Query(params): Query<Search>) -> Json<Search> {
    //    let lang = params.language;
    //     let tl = params.type_lan;

    // let user_data_search = params; // ya murio la variable
    Json(params)
}

async fn hello() -> &'static str {
    "Hello World!"
}

async fn login(Json(login): Json<Login>) -> Json<ResponseLogin> {
    let mut respone = ResponseLogin {
        status: "".to_string(),
        descripcion: "".to_string(),
    };

    if login.email == "dani@uwu.com" && login.password == "123" {
        respone = ResponseLogin {
            status: "200 OK".to_string(),
            descripcion: "Logeado exitosamente".to_string(),
        };
    } else {
        respone = ResponseLogin {
            status: "409 Conflict".to_string(),
            descripcion: "El usuario no existe".to_string(),
        }
    }

    Json(respone)
}

//solo se utiliza cuando voy a
//recibir un Json
#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}

//serialize solo se utiliza
// cuando voy a devolver un json
#[derive(Serialize)]
struct ResponseLogin {
    status: String,
    descripcion: String,
}

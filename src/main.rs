use axum::{
    extract::{Path, Query},
    http::{
        header::{ACCEPT, CONTENT_TYPE, ORIGIN},
        Method, StatusCode,
    },
    routing::{get, post},
    Json, Router,
};

use tower_http::cors::CorsLayer;

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

#[derive(Deserialize, Serialize)]
struct UuidLibro {
    id: String,
}
#[tokio::main]

// info que no puede ser vista por el user

//get para obtener informacion que quiera mostrar al usuario
async fn main() {
    let origins = [
        "http://localhost:3000".parse().unwrap(),
        "http://localhost:3000/".parse().unwrap(),
    ];

    let app = Router::new()
        .route("/", get(hello))
        .route("/login", post(login))
        .route("/libros/:id", get(libros))
        .route("/repos", get(repositorios))
        .route("/lista/libros", get(get_libros))
        .route("/libros/one", post(get_libro_uuid))
        .route("/insert", post(insert_libros))
        .route("/update", post(update_libro))
        .route("/delete", post(deleleLibro))
        .route("/insertar", post(insertar))
        .route("/actualizar", post(update))
        .route("/eliminar", post(delelete))
        .route("/select", get(select))
        .route("/select/one", post(selectOn))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_headers([ORIGIN, ACCEPT, CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        );

    axum::Server::bind(&"0.0.0.0:7000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn delelete(Json(payload): Json<UuidLibro>) -> Json<UpResponse> {
    let pool = DB::connection().await;
    let uuid = Uuid::parse_str(&payload.id).expect("error al transformar uuid");

    let sql = sqlx::query!("DELETE FROM repolib WHERE id = $1", uuid)
        .execute(&pool)
        .await
        .expect("error al eliminar libro")
        .rows_affected();

    let response = if sql > 0 {
        UpResponse {
            status: "200 OK".to_string(),
            rows_affected: true,
            description: "libro eliminado".to_string(),
        }
    } else {
        UpResponse {
            status: "404 Not Found".to_string(),
            rows_affected: false,
            description: "libro no eliminado".to_string(),
        }
    };

    Json(response)
}

struct SelectLibros {
    id: Uuid,
    titulo: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct Response {
    status: String,
    data: Vec<SelectToString>,
    description: String,
}
#[derive(Serialize)]
struct SelectToString {
    Uuid: String,
    titulo: String,
    description: String,
}
async fn select() -> Json<Response> {
    let pool = DB::connection().await;

    let sql = sqlx::query_as!(
        SelectLibros,
        "
        SELECT * from repolib
        "
    )
    .fetch_all(&pool)
    .await;

    let response = match sql {
        Ok(data) => Response {
            status: "200 OK".to_string(),
            data: data
                .into_iter()
                .map(|x| SelectToString {
                    Uuid: x.id.to_string(),
                    titulo: x.titulo,
                    description: option_to_string(x.description),
                })
                .collect(),
            description: "libros encontrados".to_string(),
        },
        Err(_err) => Response {
            status: StatusCode::CONFLICT.to_string(),
            data: vec![],
            description: _err.to_string(),
        },
    };

    Json(response)
}

#[derive(Serialize)]
struct SelectOne {
    status: String,
    data: UpdateLibros,
    description: String,
}
async fn selectOn(Json(payload): Json<UuidLibro>) -> Json<SelectOne> {
    let pool = DB::connection().await;

    let uuid = Uuid::parse_str(&payload.id).expect("error al transformar uuid");

    let sql = sqlx::query!(
        "
        SELECT * from repolib where id = $1
        ",
        uuid
    )
    .fetch_one(&pool)
    .await
    .expect("error en el registro");

    let response = SelectOne {
        status: StatusCode::OK.to_string(),
        data: UpdateLibros {
            uuid: sql.id.to_string(),
            titulo: sql.titulo,
            description: option_to_string(sql.description),
        },
        description: "libro encontrado".to_string(),
    };

    Json(response)
}

async fn _option_to_string(value: Option<String>) -> String {
    let new_value = match value {
        Some(val) => val,
        None => "".to_string(),
    };

    new_value
}

fn option_to_string(description: Option<String>) -> String {
    let resultado = match description {
        Some(desc) => desc,
        None => "".to_string(),
    };

    resultado
}
#[derive(Deserialize)]
struct InsertarLibros {
    titulo: String,
    description: String,
}

#[derive(Deserialize, Serialize)]
struct UpdateLibros {
    uuid: String,
    titulo: String,
    description: String,
}

#[derive(Serialize)]
struct InResponse {
    status: String,
    uuid: String,
    description: String,
}
async fn insertar(Json(payload): Json<InsertarLibros>) -> Json<InResponse> {
    let pool = DB::connection().await;
    let sql = sqlx::query!(
        r#"
        INSERT INTO repolib (titulo, description)
         values ($1, $2) returning id "#,
        payload.titulo,
        payload.description
    )
    .fetch_one(&pool)
    .await;

    let uuid = match sql {
        Ok(id) => id.id.to_string(),
        Err(_error) => "".to_string(),
    };

    // let response = InResponse{
    //     status: "200 OK".to_string(),
    //     uuid: uuid,
    //     description: "Query realizado".to_string(),
    // };

    Json(InResponse {
        status: "200 OK".to_string(),
        uuid: uuid,
        description: "Query realizado".to_string(),
    })
}

#[derive(Deserialize)]
struct UpdateLibross {
    uuid: String,
    titulo: String,
    description: String,
}

#[derive(Serialize)]
struct UpResponse {
    status: String,
    rows_affected: bool,
    description: String,
}

async fn update(Json(payload): Json<UpdateLibross>) -> Json<UpResponse> {
    let pool = DB::connection().await;

    let uuid = Uuid::parse_str(&payload.uuid).expect("error al transformar uuid");
    let sql = sqlx::query!(
        r#"
    UPDATE repolib
    SET titulo = $1, description = $2
    WHERE id = $3
    "#,
        payload.titulo,
        payload.description,
        uuid
    )
    .execute(&pool)
    .await
    .expect("error al actualizar")
    .rows_affected();

    let response = if sql > 0 {
        UpResponse {
            status: "200 OK".to_string(),
            rows_affected: true,
            description: "Update realizado".to_string(),
        }
    } else {
        UpResponse {
            status: "409 CONFLICT".to_string(),
            rows_affected: false,
            description: "Update no realizado".to_string(),
        }
    };

    Json(response)
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
<<<<<<< HEAD
=======
fn option_to_string(descripcion: Option<String>) -> String {
    let resultado = match descripcion {
        Some(desc) => desc,
        None => "".to_string(),
    };

    resultado
}
>>>>>>> 0a6a705c298c9d6be3176c2d20ac69502da28e18

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

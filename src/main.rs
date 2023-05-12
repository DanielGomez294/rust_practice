


use axum::{extract::{Path, Query} ,Router, Json, http::StatusCode, routing::{get,post}};
use serde::{Serialize, Deserialize};
mod database;

use database::connection::DB;
use sqlx::types::Uuid;

#[tokio::main]
async fn main() {

    let app = Router::new()
    .route("/", get(hello))
    .route("/login", post(login))
   .route("/libros/:id", get(libros))
   .route("/repos", get(repositorios))
  .route("/lista/libros", get(get_libros))
  .route("/libros/one", post(get_libro_uuid));


    axum::Server::bind(&"0.0.0.0:7000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();

}


struct LibrosDb{ 
    id: Uuid,
    titulo: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct  LibrosDBV2{
    uuid: String,
    titulo: String,
    description: String,
}


#[derive(Serialize)]
struct DefaultResponse {
    status: String,
    libro: Vec<LibrosDBV2>,
    description: String
}

struct LibroResumen{
    titulo: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct  UuidLibro{
    id: String
}

async fn get_libro_uuid(Json(payload): Json<UuidLibro>) -> Json<DefaultResponse>{
    let pool = DB::connection().await;

    let id = Uuid::parse_str(&payload.id).expect("error al transformar uuid");
    let sql = sqlx::query_as!(
        LibroResumen,
         r#"SELECT  titulo, description from repolib WHERE id = $1"#, id)
        .fetch_one(&pool)
        .await;

    let response = match sql {
        Ok(res) => DefaultResponse{
            status: "200 OK".to_string(),
            libro: vec![],
            description: "registro obtenido".to_string()
        
    },
    Err(_err) => {
        DefaultResponse{
            status: "404 Not Found".to_string(),
            libro: vec![],
            description: "libro no encontrado".to_string()
        }
    }
};
Json(response)

}

async fn get_libros() -> Json<DefaultResponse>{

    let pool = DB::connection().await;
    let libros =  sqlx::query_as!(
            LibrosDb,
           r#"SELECT * FROM repolib"#
        ).fetch_all(&pool)
        .await;

    let response = match libros{
        Ok(res )=> DefaultResponse{
        status: "200 OK".to_string(),
        libro: res.into_iter().map(|x| LibrosDBV2{
            uuid: x.id.to_string(),
            titulo: x.titulo,
            description: option_to_string(x.description)
        }).collect(),
        description: "Registros obtenidos".to_string(),
        }, 
        Err(_err) => DefaultResponse{
            status: "409 conflict".to_string(),
            libro: vec![],
            description: "error al obtener registros".to_string(),
        },
    };


   
        Json(response)
    //las funciones asincronas devuelven un future
    //await extraer la informacion que contenia esa funcion asincona
}
fn option_to_string(description: Option<String>) -> String {
    let resultado = match description  {
        Some(desc) => desc,
        None => "".to_string()
    };

    resultado
}

// https://0.0.0.0libros/:id
#[derive(Serialize)]
struct ResponseLibros{
    id: String,
    status: String,
    description: String,
}

async fn libros(Path(id): Path<String>) -> Json<ResponseLibros> {

    let libro_id = id;
    let sql = format!("SELECT * FROM libros where id = {libro_id}");
   
   let response = ResponseLibros{

        id: libro_id,
        status: "OK".to_string(),
        description: "Libro".to_string(),
 
 
   };


   Json(response)
}

#[derive( Serialize,Deserialize)]
struct Search{
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
  let mut respone = ResponseLogin{
    status: "".to_string(),
    description: "".to_string()
  };
  
   if login.email == "dani@uwu.com" && 
    login.password ==  "123"{
        respone = ResponseLogin{
            status: "200 OK".to_string(),
            description: "Logeado exitosamente".to_string()
          };
}else {
    respone = ResponseLogin{
        status: "409 Conflict".to_string(),
        description: "El usuario no existe".to_string()
      }
}

Json(respone)
}
    

//solo se utiliza cuando voy a 
//recibir un Json 
#[derive( Deserialize)]
struct Login{
    email: String,
    password: String
}


//serialize solo se utiliza
// cuando voy a devolver un json
#[derive(Serialize)]
struct  ResponseLogin{
    status: String,
    description: String,
}

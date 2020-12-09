use actix::prelude::*;
use actix_files::{Files, NamedFile};
use actix_web::{get, post, web, App, HttpServer, Result};
use std::time;

mod count_actor;
use count_actor::{CountActor, MsgIncrement};

// ---- Apis ("/api/*") ----

#[post("send-message")]
async fn send_message(
    state: web::Data<State>,
    request_data: web::Json<shared::SendMessageRequestBody>,
) -> Result<web::Json<shared::SendMessageResponseBody>> {
    Ok(web::Json(shared::SendMessageResponseBody {
        ordinal_number: state
            .count_actor
            .send(MsgIncrement)
            .await
            .expect("send MsgIncrement"),
        text: request_data.text.clone(),
    }))
}

#[get("delayed-response/{delay}")]
async fn delayed_response(delay: web::Path<u64>) -> String {
    futures_timer::Delay::new(time::Duration::from_millis(*delay)).await;
    format!("Delay was set to {}ms.", delay)
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("./client/index.html")?)
}

struct State {
    count_actor: Addr<CountActor>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let count_actor_addr = CountActor(0).start();
    HttpServer::new(move || {
        App::new()
            .data(State {
                count_actor: count_actor_addr.clone(),
            })
            .service(
                web::scope("/api/")
                    .service(send_message)
                    .service(delayed_response)
                    .default_service(web::route().to(web::HttpResponse::NotFound)),
            )
            .service(Files::new("/pkg", "./client/pkg"))
            .default_service(web::get().to(index))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

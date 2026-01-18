use actix_web::{web, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::cookie::time::Duration;
use sqlx::MySqlPool;
use uuid::Uuid;
use macros::controller;
use crate::auth::{get_logged_user, hash_password, verify_password};
use crate::blocked::{blogUserDTO, BlockedRepository, BlockedUsers};
use crate::messages::{sendMessageDTO, MessageEntity, MessageRepository};
use crate::session::SessionRepository;
use crate::users::{RegisterDTO, UserEntity, UserLoginDTO, UserRepository};
use crate::utils::{get_current_timestamp, is_user_blocked};

pub struct AppController;

#[controller("/api")]
impl AppController {

    #[postMapping("/register")]
    pub async fn register(pool: web::Data<MySqlPool>, dto: web::Json<RegisterDTO>) -> impl Responder {
        if dto.password != dto.confirm_password {
            return HttpResponse::BadRequest().body("Heslá sa nezhodujú");
        }

        let password = hash_password(&dto.password).unwrap();
        let user = UserEntity {
            id: 0,
            username: dto.username.clone(),
            password: password,
        };
        user.save(&pool).await.unwrap();
        HttpResponse::Ok().body("Uživateľ bol úspešne zaregistrovaný")
    }

    #[postMapping("/login")]
    pub async fn login(pool: web::Data<MySqlPool>, dto: web::Json<UserLoginDTO>) -> impl Responder {
        let user = match UserRepository::find_by_username(&pool, dto.username.clone()).await {
            Ok(Some(u)) => u,
            _ => return HttpResponse::Unauthorized().body("Nesprávne meno alebo heslo"),
        };

        if !verify_password(&dto.password, &user.password) {
            return HttpResponse::Unauthorized().body("Nesprávne meno alebo heslo");
        }
        let mut session_token: Uuid;

        loop {
            session_token = Uuid::new_v4();
            let check_session = SessionRepository::find_by_token(&pool, session_token.to_string()).await.unwrap();
            if check_session.is_none() {
                break;
            }
        }

        let session = crate::session::SessionEntity {
            id: 0,
            user_id: user.id,
            token: session_token.to_string(),
            created_at: get_current_timestamp(),
        };

        session.save(&pool).await.unwrap();


        let cookie = Cookie::build("user_cookies", session_token.to_string())
            .path("/")
            .http_only(true)
            .same_site(SameSite::Strict)
            .max_age(Duration::days(1))
            .finish();

        HttpResponse::Ok()
            .cookie(cookie)
            .body(format!("Uživateľ {} sa úspešne prihlásil", user.username))
    }

    #[postMapping("/logout")]
    pub async fn logout(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest) -> impl Responder {
        let cookie = req.cookie("user_cookies");
        if cookie.is_none() {
            return HttpResponse::BadRequest().body("Žiadna aktívna session");
        }

        let session_token = cookie.unwrap().value().to_string();
        SessionRepository::delete_by_token(&pool, session_token).await.expect("Nepodarilo sa odstrániť session");

        let cookie = Cookie::build("user_cookies", "")
            .path("/")
            .http_only(true)
            .same_site(SameSite::Strict)
            .max_age(Duration::ZERO)
            .finish();

        HttpResponse::Ok()
            .cookie(cookie)
            .body("Uživateľ bol úspešne odhlásený")
    }

    #[getMapping("/check-session")]
    pub async fn check_session(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest) -> impl Responder {

        let user = match get_logged_user(&pool, req).await {
            Some(u) => return HttpResponse::Ok().body(format!("Prihlásený ako: {}", u.username)),
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený. Prosím, prihlás sa."),
        };

    }

    #[postMapping("/send/{id}")]
    pub async fn send_message(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest, id: web::Path<i64>, dto: web::Json<sendMessageDTO>) -> impl Responder {
        let user = match get_logged_user(&pool, req.clone()).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený. Prosím, prihlás sa."),
        };

        let reciever: UserEntity = match UserRepository::find_by_id(&pool, *id).await.unwrap() {
            Some(u) => u,
            _ => return HttpResponse::NotFound().body("Používateľ nebol nájdený."),
        };

        if user.id == reciever.id {
            return HttpResponse::BadRequest().body("Nemôžeš si posielať správy sám sebe.");
        }

        let blocked_by_me = is_user_blocked(&pool, user.id, reciever.id).await;
        let blocked_by_them = is_user_blocked(&pool, reciever.id, user.id).await;

        if blocked_by_me || blocked_by_them {
            return HttpResponse::Forbidden().body("Užívateľ sa nenašiel.");
        }

        let messageEntity: MessageEntity = MessageEntity {
            id: 0,
            sender_id: user.id,
            receiver_id: *id,
            content: dto.content.clone(),
            sent_time: get_current_timestamp(),
        };

        messageEntity.save(&pool).await.unwrap();
        return HttpResponse::Ok().body(format!("Správa bola odoslaná používateľovi"));
    }

    #[postMapping("/block-user")]
    pub async fn block_user(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest, dto: web::Json<blogUserDTO>) -> HttpResponse {
        let user = match get_logged_user(&pool, req.clone()).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený. Prosím, prihlás sa."),
        };

        let target: UserEntity = match UserRepository::find_by_id(&pool, dto.user_id).await.unwrap() {
            Some(u) => u,
            _ => return HttpResponse::NotFound().body("Používateľ nebol nájdený."),
        };

        if user.id == target.id {
            return HttpResponse::BadRequest().body("Nemôžeš zablokovať sám seba.");
        }

        let possible_block = is_user_blocked(&pool, user.id, target.id).await;
        if possible_block {
            return HttpResponse::BadRequest().body("Používateľ je už zablokovaný.");
        }

        let blockedEntity: BlockedUsers = BlockedUsers {
            id: 0,
            user_id: user.id,
            blocked_user_id: target.id,
        };

        blockedEntity.save(&pool).await.unwrap();
        HttpResponse::Ok().body(format!("Používateľ {} bol zablokovaný", target.username))
    }

    #[postMapping("/unblock-user")]
    pub async fn unblock_user(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest, dto: web::Json<blogUserDTO>) -> HttpResponse {
        let user = match get_logged_user(&pool, req.clone()).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený. Prosím, prihlás sa."),
        };

        let target: UserEntity = match UserRepository::find_by_id(&pool, dto.user_id).await.unwrap() {
            Some(u) => u,
            _ => return HttpResponse::NotFound().body("Používateľ nebol nájdený."),
        };

        if user.id == target.id {
            return HttpResponse::BadRequest().body("Nemôžeš odblokovať sám seba.");
        }

        let possible_block = is_user_blocked(&pool, user.id, target.id).await;
        if !possible_block {
            return HttpResponse::BadRequest().body("Používateľ nie je zablokovaný.");
        }

        match BlockedRepository::unblock_user(&pool, user.id, target.id).await {
            Ok(_) => HttpResponse::Ok().body(format!("Používateľ {} bol odblokovaný", target.username)),
            Err(_) => HttpResponse::InternalServerError().body("Nepodarilo sa odblokovať používateľa"),
        }

    }


    #[getMapping("/chat/{id}")]
    pub async fn get_chat_history(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest, id: web::Path<i64>) -> impl Responder {
        let user = match get_logged_user(&pool, req).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený."),
        };

        let target_id = *id;

        match MessageRepository::get_chat_history(&pool, user.id, target_id, target_id, user.id).await {
            Ok(messages) => {
                HttpResponse::Ok().json(messages)
            },
            Err(e) => {
                HttpResponse::InternalServerError().body("Chyba databázy")
            }
        }
    }


    #[getMapping("/users")]
    pub async fn list_users(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest) -> impl Responder {
        let user = match get_logged_user(&pool, req).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený."),
        };

        match UserRepository::find_all_except_me(&pool, user.id).await {
            Ok(users) => HttpResponse::Ok().json(users),
            Err(e) => {
                HttpResponse::InternalServerError().body("Chyba databázy")
            }
        }
    }

    #[deleteMapping("/message/{id}")]
    pub async fn delete_message(pool: web::Data<MySqlPool>, req: actix_web::HttpRequest, message_id: web::Path<i64>) -> impl Responder {
        let user = match get_logged_user(&pool, req).await {
            Some(u) => u,
            _ => return HttpResponse::Unauthorized().body("Nie si prihlásený."),
        };

        let message = match MessageRepository::find_by_id(&pool, *message_id).await.unwrap() {
            Some(m) => m,
            _ => return HttpResponse::NotFound().body("Správa nebola nájdená."),
        };

        if message.sender_id != user.id {
            return HttpResponse::Forbidden().body("Nemôžeš odstrániť túto správu.");
        }

        match MessageRepository::delete_by_id(&pool, *message_id).await {
            Ok(_) => HttpResponse::Ok().body("Správa bola úspešne odstránená."),
            Err(_) => HttpResponse::InternalServerError().body("Nepodarilo sa odstrániť správu."),
        }
    }

}
use actix_web::web::{Data, Json, Path};
use actix_web::{Error, HttpResponse, Responder};
use apistos::actix::CreatedJson;
use apistos::{api_operation, ApiComponent};
use chrono::{DateTime, NaiveDateTime, Utc};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha512};
use sqlx::{query, FromRow, PgPool};
use std::fs;
use uuid::Uuid;
/* TODO: Auth
use oauth2::{
  AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, Scope, TokenResponse, RedirectUrl,
};
use oauth2::{ClientId, ClientSecret, AuthorizationCode, TokenResponse, OAuth2Error};
use std::env;

#[derive(Debug)]
struct OAuthConfig {
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: String,
}

impl OAuthConfig {
    fn from_env() -> Self {
        dotenv::dotenv().ok(); // Load .env file
        Self {
            client_id: ClientId::new(env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set")),
            client_secret: ClientSecret::new(env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set")),
            redirect_uri: env::var("REDIRECT_URI").expect("REDIRECT_URI must be set"),
        }
    }
}
async fn google_login() -> impl Responder {
  let config = OAuthConfig::from_env();

  let auth_url = config.client_id.auth_url(
      RedirectUrl::new(config.redirect_uri.clone()).unwrap(),
      Some(CsrfToken::new_random()), // CSRF protection
      vec![Scope::new("profile".to_string()), Scope::new("email".to_string())],
  );

  HttpResponse::Found()
      .append_header(("Location", auth_url.to_string()))
      .finish()
}

async fn google_callback(code: Query<(String,)>) -> impl Responder {
  let config = OAuthConfig::from_env();

  // Exchange the authorization code for an access token
  let token = match config.client_id.exchange_code(AuthorizationCode::new(code.0.clone())) {
      Ok(token) => token,
      Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {:?}", e)),
  };

  // Use the token to access Google APIs to get user info
  // (You would need to implement this part)

  // For demonstration purposes, we just return the token
  HttpResponse::Ok().body(format!("Access Token: {:?}", token.access_token()))
}
*/

// JsonSchema !
#[derive(Deserialize, JsonSchema, ApiComponent)] // <------------- `JsonSchema`, `ApiComponent` are Apistos Traits
pub struct NewTodo {
    pub title: String,
    pub description: Option<String>,
}

// JsonSchema !
#[derive(Serialize, JsonSchema, ApiComponent)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
}

// #[api_operation(
//   tag = "pet",
//   summary = "Add a new pet to the store",
//   description = r###"Add a new pet to the store
//     Plop"###,
//   error_code = 405
// )]
#[api_operation(summary = "Get an element from the todo list")]
pub(crate) async fn get_todo(todo_id: Path<Uuid>) -> Result<Json<Todo>, Error> {
    // <----- scope("/test").service(scope("/todo").service(resource("/{todo_id}") ... the param is defined as a Uuid
    // because it is a sample app, we do not currently implement any logic to store todos
    Ok(Json(Todo {
        id: todo_id.into_inner(),
        title: "some title".to_string(),
        description: None,
    }))
}
// Use `String` for string param
//
// // extract path info from "/{name}/{count}/index.html" into tuple
// // {name}  - deserialize a String
// // {count} - deserialize a u32
// #[get("/{name}/{count}/index.html")]
// async fn index(path: Path<(String, u32)>) -> String {

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn add_todo(
    pool: Data<PgPool>,
    req: Json<NewTodo>,
) -> Result<CreatedJson<Todo>, Error> {
    // <----- it should eat both a `Path` (for parameters) and a `Json` (for body)
    let new_todo = req.into_inner();
    Ok(CreatedJson(Todo {
        id: Uuid::new_v4(),
        title: new_todo.title,
        description: new_todo.description,
    }))
}

#[derive(FromRow, Debug, Serialize, JsonSchema, ApiComponent)]
pub struct Blog {
    pub publishedtime: NaiveDateTime,
    pub minread: i32,
    pub headerimage: Option<String>,
    pub title_en: String,
    pub permalink: String,
    pub richtextcontent_en: String,
    pub tags_en: String,
}

#[derive(Debug, Serialize, JsonSchema, ApiComponent)]
pub struct Out {
    pub maxPageNum: i32,
    pub posts: Vec<Blog>,
}

#[derive(Debug, Serialize, JsonSchema, ApiComponent, FromRow)]
pub struct Proj {
    pub detailslink: String,
    pub title_en: String,
    pub description_en: String,
    pub thumbnailbase64: Option<String>,
}

#[api_operation(summary = "Get Alex's Blogs by Page Number")]
pub(crate) async fn getBlogs(
    pool: Data<PgPool>,
    page: Path<i64>,
) -> Result<Json<Out>, Error> {
    let pool_ref = pool.get_ref();

    // Don't use the macro version `query_as!` (slow and does not use the `from_row` trait !) !!!
    let offset = 3 * (page.into_inner() - 1);
    let rows = sqlx::query_as::<_, Blog>(
        r#"SELECT * FROM blog ORDER BY publishedtime DESC OFFSET $1 ROWS FETCH FIRST 3 ROWS ONLY"#,
    )
    .bind(offset)
    .fetch_all(pool_ref)
    .await
    .map_err(|e| {
        // Log the error or handle it as necessary
        eprintln!("Database query failed: {}", e);
        // Return an Actix Web error response
        actix_web::error::ErrorInternalServerError("Failed to fetch blogs")
    })?;

    // If successful, return the results
    let max_page_num = query!("select CAST(CEIL(COUNT(*) / 3.0) AS INTEGER) from blog")
        .fetch_one(pool_ref)
        .await
        .map_err(|e| {
            // Log the error or handle it as necessary
            eprintln!("Database query failed: {}", e);
            // Return an Actix Web error response
            actix_web::error::ErrorInternalServerError("Failed to fetch count")
        })
        .unwrap()
        .ceil
        .unwrap();

    let out = Out {
        maxPageNum: max_page_num,
        posts: rows,
    };
    Ok(Json(out))
}

#[api_operation(summary = "Get one Alex's Blog by Permalink")]
pub(crate) async fn getBlogPermalink(pool: Data<PgPool>,
    slug: Path<String>,) -> Result<Json<Blog>, Error> {
    let rows = sqlx::query_as::<_, Blog>(
        r#"SELECT * FROM blogs WHERE permalink = $1"#,
    )
    .bind(slug.into_inner())
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        // Log the error or handle it as necessary
        eprintln!("Database query failed: {}", e);
        // Return an Actix Web error response
        actix_web::error::ErrorInternalServerError("No such blog")
    })?;
    
    Ok(Json(rows))
}

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn signIn_github(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn signIn_google(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[derive(Debug)]
pub struct EmailParams {
    pub lang: String,
    pub email_type: String,
    pub receiver_display_name: String,
    pub receiver_email: String,
    pub token: String,
}

pub async fn send_mail(params: EmailParams) {
    // SMTP server details
    let sender_email = "Peciel <noreply@peciel.com>";

    // Determine the subject based on the language and email type
    let (topic, filelang) = match params.lang.as_str() {
        "en" => match params.email_type.as_str() {
            "confirmAccount" => ("Please Confirm Your Email", "en"),
            "resetPassword" => ("Reset Your Password", "en"),
            _ => ("Welcome", "en"),
        },
        "zh_cn" => match params.email_type.as_str() {
            "confirmAccount" => ("请确认您的电邮", "hans"),
            "resetPassword" => ("重设您的密码", "hans"),
            _ => ("欢迎", "hans"),
        },
        "zh_hk" => match params.email_type.as_str() {
            "confirmAccount" => ("請確認您的電郵", "hant"),
            "resetPassword" => ("重設您的密碼", "hant"),
            _ => ("歡迎", "hant"),
        },
        _ => ("Welcome", "en"), // Default fallback to English
    };

    // Read the HTML template
    let template_path = format!("email_server/{}_{}.html", params.email_type, filelang);
    let html_template = fs::read_to_string(template_path).expect("Error reading email template");

    // Replace placeholders in the template with actual values
    let html_content = html_template
        .replace("{{username}}", &params.receiver_display_name)
        .replace("{{token}}", &params.token);

    // Construct the email message
    let email = Message::builder()
        .from(sender_email.parse().unwrap())
        .reply_to(sender_email.parse().unwrap())
        .to(params.receiver_email.parse().unwrap())
        .subject(format!("{} - Peciel", topic))
        .header(ContentType::TEXT_HTML)
        .body(html_content)
        .unwrap();

    let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

    // TODO: Change gmail to local...
    let smtp_server = "peciel.com";
    let port = 20005;
    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }
}

#[derive(Debug, Deserialize, JsonSchema, ApiComponent)]
pub struct SignUpRequest {
    email: String,
    displayname: String,
    password: String,
}

// #[post("/signUp")]
#[api_operation(
    summary = "Account registration. This will send confirmation email to your email. Check your mailbox."
)]
pub(crate) async fn signUp(pool: Data<PgPool>, req: Json<SignUpRequest>) -> impl Responder {
    print!("{:?}", req);

    let email = &req.email;
    let displayname = &req.displayname;
    let lang = "en";

    // Check if the user already exists
    let row = query!(
        "SELECT email FROM users WHERE email = $1 AND confirmMailToken IS NULL",
        email
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    if !row.is_empty() {
        return HttpResponse::Ok().json(json!({"err": "duplicate"}));
    }

    print!("Sending confirm email to: {}", email);

    // Generate confirmMailToken and salt
    let confirm_mail_token: String = format!("{:030x}", rand::thread_rng().gen_range(0..u128::MAX));
    let salt: String = format!("{:030x}", rand::thread_rng().gen_range(0..u128::MAX));

    // Hash the password with the salt
    let mut hasher = Sha512::new();
    hasher.update(format!("{}{}", salt, req.password));
    let hash = format!("{:x}", hasher.finalize());

    // Check if the user email already exists
    let existing_user = query!("SELECT email FROM users WHERE email = $1", email)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default();

    if existing_user.is_empty() {
        query!("INSERT INTO users (email, displayname, salt, hash, confirmMailToken) VALUES ($1, $2, $3, $4, $5)",
            email, displayname, salt, hash, confirm_mail_token)
            .execute(pool.get_ref())
            .await.unwrap();
    } else {
        query!("UPDATE users SET displayname = $1, salt = $2, hash = $3, confirmMailToken = $4 WHERE email = $5",
            displayname, salt, hash, confirm_mail_token, email)
            .execute(pool.get_ref())
            .await.unwrap();
    }
    let params = EmailParams {
        lang: lang.to_string(),
        email_type: "confirmAccount".to_string(),
        receiver_display_name: displayname.to_string(),
        receiver_email: email.to_string(),
        token: confirm_mail_token.to_string(),
    };

    send_mail(params).await;

    HttpResponse::Ok().json(json!({"ok": 1}))
}

#[api_operation(
    summary = "Activate an account using the one-time token included in the confirmation email. Log in email and password are not needed."
)]
pub(crate) async fn confirmAccount(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Log in.")]
pub(crate) async fn signIn(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Log out.")]
pub(crate) async fn signOut(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Request a password reset.")]
pub(crate) async fn requestChangePassword(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(
    summary = "(Actually) change user password after requesting a reset using the one-time token included in the request email. The log in email is not needed."
)]
pub(crate) async fn changePasswordWithToken(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn setAccountSetting(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn getProjects(pool: Data<PgPool>) -> Result<Json<Vec<Proj>>, Error> {
    let rows = sqlx::query_as::<_, Proj>("SELECT * FROM projects")
        .fetch_all(pool.get_ref())
        .await
        .map_err(|e| {
            // Log the error or handle it as necessary
            eprintln!("Database query failed: {}", e);
            // Return an Actix Web error response
            actix_web::error::ErrorInternalServerError("Failed to fetch")
        })?;

    // If successful, return the results
    Ok(Json(rows))
}

#[api_operation(summary = "Add a new element to the todo list")]
pub(crate) async fn converted(
    pool: Data<PgPool>,
    req: Json<Option<u8>>,
) -> Result<Json<Option<u8>>, Error> {
    Ok(Json(None))
}

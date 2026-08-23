use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage {
    error_message: Option<String>,
}

async fn login_page() -> Result<Html<String>, AppError> {
    render_login_page(None)
}

fn render_login_page(error_message: Option<&str>) -> Result<Html<String>, AppError> {
    let html = LoginPage {
        error_message: error_message.map(str::to_owned),
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterPage {
    error_message: Option<String>,
    tried_username: Option<String>,
}

async fn register_page() -> Result<Html<String>, AppError> {
    render_register_page(None, None)
}

fn render_register_page(
    error_message: Option<&str>,
    tried_username: Option<&str>,
) -> Result<Html<String>, AppError> {
    let html = RegisterPage {
        error_message: error_message.map(str::to_owned),
        tried_username: tried_username.map(str::to_owned),
    }
    .render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<Response, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => {
            return Ok(render_login_page(Some("user not found."))?.into_response());
        }
        Err(AppError::InvalidCredentials) => {
            return Ok(render_login_page(Some("invalid credentials."))?.into_response());
        }
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/")).into_response())
}

async fn register(
    repository: Repository,
    Form(request): Form<LoginForm>,
) -> Result<Response, AppError> {
    let LoginForm { username, password } = request;
    let unauth_user = UnauthenticatedUser::new(username.clone(), password);
    match unauth_user.register(&repository).await {
        Ok(_user) => Ok(Redirect::to("/login").into_response()),
        Err(AppError::UsernameTaken) => Ok(render_register_page(
            Some("username is already taken"),
            Some(&username),
        )?
        .into_response()),
        Err(other_error) => Err(other_error),
    }
}

async fn index(maybe_user: Option<User>) -> Result<Response, AppError> {
    match maybe_user {
        Some(user) => Ok(Html(format!("Hello, {}", user.username())).into_response()),
        None => Ok(Redirect::to("/login").into_response()),
    }
}

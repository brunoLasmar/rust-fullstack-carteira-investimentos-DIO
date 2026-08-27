use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;
use tokio::try_join;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::{Asset, OwnedAsset, PortfolioSummary},
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/register", get(register_page).post(register))
        .route("/logout", get(logout))
        .route("/assets", get(assets).post(purchase_asset))
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

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (jar.remove("token"), Redirect::to("/login"))
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

#[derive(Template)]
#[template(path = "assets.html")]
pub struct AssetsPage {
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    portfolio_summary: PortfolioSummary,
    error_message: Option<String>,
    user: User,
}

pub async fn assets(repository: Repository, user: User) -> Result<Html<String>, AppError> {
    render_assets_page(repository, user, None).await
}

async fn render_assets_page(
    repository: Repository,
    user: User,
    error_message: Option<String>,
) -> Result<Html<String>, AppError> {
    let (owned_assets, available_assets, portfolio_summary) = try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets(),
        repository.portfolio_summary(user.id())
    )?;

    let html = AssetsPage {
        owned_assets,
        available_assets,
        portfolio_summary,
        error_message,
        user,
    }
    .render()?;

    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct PurchaseAssetForm {
    asset_id: i64,
    unit_value: f64,
    quantity: f64,
}

pub async fn purchase_asset(
    repository: Repository,
    user: User,
    Form(request): Form<PurchaseAssetForm>,
) -> Result<Response, AppError> {
    if !request.quantity.is_finite() || request.quantity <= 0.0 {
        return Ok(render_assets_page(
            repository,
            user,
            Some("quantity must be greater than zero".to_owned()),
        )
        .await?
        .into_response());
    }

    match repository
        .insert_owned_asset(
            user.id(),
            request.asset_id,
            request.quantity,
            request.unit_value,
        )
        .await
    {
        Ok(()) => Ok(Redirect::to("/assets").into_response()),
        Err(error) => {
            let error = AppError::Database(error);
            tracing::error!(error = ?error, "purchase registration failed");
            Ok(
                render_assets_page(repository, user, Some(error.public_message().to_owned()))
                    .await?
                    .into_response(),
            )
        }
    }
}

pub mod filters {
    use askama;
    use time::{
        OffsetDateTime, format_description::StaticFormatDescription, macros::format_description,
    };

    #[askama::filter_fn]
    pub fn decimal(value: &f64, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("{value:.2}"))
    }

    #[askama::filter_fn]
    pub fn percentage(value: &f64, _env: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("{value:.2}"))
    }

    #[askama::filter_fn]
    pub fn human_datetime(
        datetime: &OffsetDateTime,
        _env: &dyn askama::Values,
    ) -> askama::Result<String> {
        const HUMAN_READABLE_FORMAT: StaticFormatDescription =
            format_description!(version = 2, "[year]-[month]-[day] [hour]:[minute]");

        datetime
            .format(HUMAN_READABLE_FORMAT)
            .map_err(askama::Error::custom)
    }
}

async fn index(maybe_user: Option<User>) -> Result<Redirect, AppError> {
    match maybe_user {
        Some(_) => Ok(Redirect::to("/assets")),
        None => Ok(Redirect::to("/login")),
    }
}

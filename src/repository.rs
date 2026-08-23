use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, OwnedAsset, PortfolioSummary, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
             VALUES ($1, $2)
             RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
             SET name=COALESCE($2, name),
                 unit_value=COALESCE($3, unit_value)
             WHERE id=$1
             RETURNING id, name, unit_value;",
            asset_id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn insert_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
        unit_value: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO owned_assets 
             (user_id, asset_id, quantity_owned, bought_for)
             VALUES ($1, $2, $3, $4);",
            user_id,
            asset_id,
            quantity,
            unit_value
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        sqlx::query_as!(
            OwnedAsset,
            r#"
            SELECT
             a.id,
             a.name,
             a.unit_value,
             SUM((a.unit_value - o.bought_for) * o.quantity_owned) AS "value_delta!",
             SUM(o.quantity_owned) AS "quantity_owned!",
             JSON_AGG(
              JSON_BUILD_OBJECT(
               'bought_at', o.timestamp,
               'bought_for', o.bought_for,
               'quantity_bought', o.quantity_owned,
               'value_delta', (a.unit_value - o.bought_for) * o.quantity_owned
              )
             ) AS "purchase_history!: _"
            FROM assets AS a
            JOIN owned_assets AS o
              ON o.asset_id = a.id
            WHERE o.user_id = $1
            GROUP BY a.id;
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn portfolio_summary(&self, user_id: i64) -> sqlx::Result<PortfolioSummary> {
        sqlx::query_as!(
            PortfolioSummary,
            r#"
            SELECT
             COALESCE(SUM(o.bought_for * o.quantity_owned), 0.0) AS "invested_value!",
             COALESCE(SUM(a.unit_value * o.quantity_owned), 0.0) AS "current_value!",
             COALESCE(SUM((a.unit_value - o.bought_for) * o.quantity_owned), 0.0) AS "total_return!",
             CASE
              WHEN COALESCE(SUM(o.bought_for * o.quantity_owned), 0.0) > 0.0
              THEN COALESCE(SUM((a.unit_value - o.bought_for) * o.quantity_owned), 0.0)
                   / SUM(o.bought_for * o.quantity_owned) * 100.0
              ELSE 0.0
             END AS "return_percentage!"
            FROM assets AS a
            JOIN owned_assets AS o
              ON o.asset_id = a.id
            WHERE o.user_id = $1;
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;",
            username,
            password_hash,
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}

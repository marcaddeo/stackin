use crate::{internal_error, AppContext, Connection};
use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    http::StatusCode,
};
use diesel_async::pooled_connection::deadpool::{Object, Pool};

pub struct DatabaseConnection(pub Object<Connection>);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync + AppContext,
    DatabasePool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let DatabasePool(pool) = DatabasePool::from_ref(state);
        let conn = pool.get().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}

struct DatabasePool(Pool<Connection>);

impl<T: AppContext> FromRef<T> for DatabasePool {
    fn from_ref(input: &T) -> Self {
        Self(input.database().clone())
    }
}

pub struct JobScheduler(pub tokio_cron_scheduler::JobScheduler);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for JobScheduler
where
    S: Send + Sync + AppContext,
    JobSchedulerInstance: FromRef<S>,
{
    type Rejection = ();

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let JobSchedulerInstance(instance) = JobSchedulerInstance::from_ref(state);
        Ok(Self(instance))
    }
}

struct JobSchedulerInstance(tokio_cron_scheduler::JobScheduler);

impl<T: AppContext> FromRef<T> for JobSchedulerInstance {
    fn from_ref(input: &T) -> Self {
        Self(input.scheduler().clone())
    }
}

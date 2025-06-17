use std::{borrow::Cow, result::Result, str::FromStr, sync::Arc};

use http::Uri;
use manifest::ManifestResolver;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status, service::Interceptor};

use crate::{
    Error,
    model::{HairpinDaemon, HairpinSource, HairpinSourceLocation},
};

use super::proto::CreateSourceResponse;
pub use super::proto::{
    CreateSourceRequest, DeleteSourceRequest, hairpin_source_service_server::*,
};

#[derive(Debug, Clone)]
pub struct Service(Arc<HairpinDaemon>);
#[tonic::async_trait]
impl HairpinSourceService for Service {
    async fn delete(&self, request: Request<DeleteSourceRequest>) -> Result<Response<()>, Status> {
        Ok(Response::new(Self::delete(&self, request).await?))
    }
    async fn create(
        &self,
        request: Request<CreateSourceRequest>,
    ) -> Result<Response<CreateSourceResponse>, Status> {
        Ok(Response::new(Self::create(&self, request).await?))
    }
}
impl Service {
    async fn delete(&self, request: Request<DeleteSourceRequest>) -> Result<(), Error> {
        let mut sources = self.0.manifests().write().await;
        for id in request.into_inner().ids {
            sources.remove(&id);
        }
        Ok(())
    }
    async fn create(
        &self,
        request: Request<CreateSourceRequest>,
    ) -> Result<CreateSourceResponse, Error> {
        let guard = request
            .extensions()
            .get::<SourceScheme>()
            .cloned()
            .unwrap_or_default();
        let mut output = Vec::new();
        for source in request
            .into_inner()
            .sources
            .into_iter()
            .map(|source| Uri::from_str(source.as_str()))
        {
            let source = source?;
            guard.validate(&source)?;
            let location: HairpinSourceLocation = source.try_into()?;
            let manifest = location.resolve().await?;
            output.push(HairpinSource::new(location, manifest));
        }
        let mut sources = self.0.manifests().write().await;
        let mut ids = Vec::new();
        for source in output {
            let id = self.0.new_id().await;
            sources.insert(id, RwLock::new(source));
            ids.push(id);
        }
        Ok(CreateSourceResponse { ids })
    }
}
#[derive(Debug, Clone, Default)]
pub struct SourceScheme<'a>(Cow<'a, [&'a str]>);
#[derive(Debug, Clone)]
pub struct SourceSchemeGuard<'a>(SourceScheme<'a>);

impl Interceptor for SourceSchemeGuard<'static> {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request.extensions_mut().insert(self.0.clone());
        Ok(request)
    }
}

impl<'a> SourceScheme<'a> {
    fn validate(&self, value: &Uri) -> Result<(), Error> {
        if value
            .scheme_str()
            .is_some_and(|scheme| self.0.contains(&scheme))
        {
            Ok(())
        } else {
            Err(Error::ProhibitedUri(value.to_string()))
        }
    }
}

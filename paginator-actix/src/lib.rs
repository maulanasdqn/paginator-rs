use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use paginator_rs::{PaginationParams, PaginatorResponse, PaginatorResponseMeta, SortDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}

impl PaginationQuery {
    pub fn into_params(self) -> PaginationParams {
        let sort_direction = self
            .sort_direction
            .and_then(|s| match s.to_lowercase().as_str() {
                "asc" => Some(SortDirection::Asc),
                "desc" => Some(SortDirection::Desc),
                _ => None,
            });

        PaginationParams {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, 100),
            sort_by: self.sort_by,
            sort_direction,
            filters: Vec::new(),
            search: None,
            disable_total_count: false,
            cursor: None,
        }
    }

    pub fn as_params(&self) -> PaginationParams {
        let sort_direction =
            self.sort_direction
                .as_ref()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "asc" => Some(SortDirection::Asc),
                    "desc" => Some(SortDirection::Desc),
                    _ => None,
                });

        PaginationParams {
            page: self.page.max(1),
            per_page: self.per_page.clamp(1, 100),
            sort_by: self.sort_by.clone(),
            sort_direction,
            filters: Vec::new(),
            search: None,
            disable_total_count: false,
            cursor: None,
        }
    }
}

#[derive(Debug)]
pub struct PaginatedJson<T> {
    response: PaginatorResponse<T>,
}

impl<T> PaginatedJson<T>
where
    T: Serialize,
{
    pub fn new(data: Vec<T>, params: &PaginationParams, total: u32) -> Self {
        Self {
            response: PaginatorResponse {
                data,
                meta: PaginatorResponseMeta::new(params.page, params.per_page, total),
            },
        }
    }

    pub fn from_response(response: PaginatorResponse<T>) -> Self {
        Self { response }
    }
}

impl<T> Responder for PaginatedJson<T>
where
    T: Serialize,
{
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut response = HttpResponse::Ok();

        if let Some(total) = self.response.meta.total {
            response.insert_header(("X-Total-Count", total.to_string()));
        }
        if let Some(total_pages) = self.response.meta.total_pages {
            response.insert_header(("X-Total-Pages", total_pages.to_string()));
        }
        response.insert_header(("X-Current-Page", self.response.meta.page.to_string()));
        response.insert_header(("X-Per-Page", self.response.meta.per_page.to_string()));

        response.json(&self.response)
    }
}

pub fn create_paginated_response<T>(
    data: Vec<T>,
    params: &PaginationParams,
    total: u32,
) -> PaginatedJson<T>
where
    T: Serialize,
{
    PaginatedJson::new(data, params, total)
}

pub mod middleware {
    use actix_web::{
        dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
        Error,
    };
    use futures_util::future::LocalBoxFuture;
    use std::future::{ready, Ready};

    pub struct PaginationMiddleware;

    impl<S, B> Transform<S, ServiceRequest> for PaginationMiddleware
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type InitError = ();
        type Transform = PaginationMiddlewareService<S>;
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(PaginationMiddlewareService { service }))
        }
    }

    pub struct PaginationMiddlewareService<S> {
        service: S,
    }

    impl<S, B> Service<ServiceRequest> for PaginationMiddlewareService<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            let fut = self.service.call(req);

            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        }
    }
}

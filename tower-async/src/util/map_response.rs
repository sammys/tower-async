use std::fmt;
use tower_async_layer::Layer;
use tower_async_service::Service;

/// Service returned by the [`map_response`] combinator.
///
/// [`map_response`]: crate::util::ServiceExt::map_response
#[derive(Clone)]
pub struct MapResponse<S, F> {
    inner: S,
    f: F,
}

impl<S, F> fmt::Debug for MapResponse<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponse")
            .field("inner", &self.inner)
            .field("f", &format_args!("{}", std::any::type_name::<F>()))
            .finish()
    }
}

/// A [`Layer`] that produces a [`MapResponse`] service.
///
/// [`Layer`]: tower_async_layer::Layer
#[derive(Debug, Clone)]
pub struct MapResponseLayer<F> {
    f: F,
}

impl<S, F> MapResponse<S, F> {
    /// Creates a new `MapResponse` service.
    pub fn new(inner: S, f: F) -> Self {
        MapResponse { f, inner }
    }

    /// Returns a new [`Layer`] that produces [`MapResponse`] services.
    ///
    /// This is a convenience function that simply calls [`MapResponseLayer::new`].
    ///
    /// [`Layer`]: tower_async_layer::Layer
    pub fn layer(f: F) -> MapResponseLayer<F> {
        MapResponseLayer { f }
    }
}

#[async_trait::async_trait]
impl<S, F: Send, Request, Response> Service<Request> for MapResponse<S, F>
where
    S: Service<Request>,
    F: Fn(S::Response) -> Response,
    for<'async_trait>Request: Send + 'async_trait,
    Self: Sync,
{
    type Response = Response;
    type Error = S::Error;

    #[inline]
    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        match self.inner.call(request).await {
            Ok(response) => Ok((self.f)(response)),
            Err(error) => Err(error),
        }
    }
}

impl<F> MapResponseLayer<F> {
    /// Creates a new [`MapResponseLayer`] layer.
    pub fn new(f: F) -> Self {
        MapResponseLayer { f }
    }
}

impl<S, F> Layer<S> for MapResponseLayer<F>
where
    F: Clone,
{
    type Service = MapResponse<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        MapResponse {
            f: self.f.clone(),
            inner,
        }
    }
}

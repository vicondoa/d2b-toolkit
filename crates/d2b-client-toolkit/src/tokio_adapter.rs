use std::{fmt, future::Future};
use tokio::{
    runtime::Handle,
    task::{JoinError, JoinHandle},
};

#[derive(Clone)]
pub struct TokioClientAdapter {
    handle: Handle,
}

impl TokioClientAdapter {
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    pub fn current() -> Result<Self, TokioAdapterError> {
        Handle::try_current()
            .map(Self::new)
            .map_err(|_| TokioAdapterError::RuntimeUnavailable)
    }

    pub fn spawn<F>(&self, future: F) -> TokioClientTask<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        TokioClientTask {
            inner: self.handle.spawn(future),
        }
    }
}

impl fmt::Debug for TokioClientAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TokioClientAdapter")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TokioAdapterError {
    #[error("a Tokio runtime is required for canonical d2b client operations")]
    RuntimeUnavailable,
    #[error("the Tokio client task did not complete")]
    TaskFailed,
}

pub struct TokioClientTask<T> {
    inner: JoinHandle<T>,
}

impl<T> TokioClientTask<T> {
    pub async fn join(self) -> Result<T, TokioAdapterError> {
        self.inner.await.map_err(map_join_error)
    }
}

impl<T> fmt::Debug for TokioClientTask<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TokioClientTask")
    }
}

fn map_join_error(_: JoinError) -> TokioAdapterError {
    TokioAdapterError::TaskFailed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_requires_an_entered_tokio_runtime() {
        assert_eq!(
            TokioClientAdapter::current().unwrap_err(),
            TokioAdapterError::RuntimeUnavailable
        );
    }

    #[tokio::test]
    async fn runs_client_work_on_the_selected_runtime() {
        let adapter = TokioClientAdapter::current().unwrap();
        assert_eq!(adapter.spawn(async { 42_u8 }).join().await.unwrap(), 42);
    }
}

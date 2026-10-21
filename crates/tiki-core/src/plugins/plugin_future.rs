use std::future::Future;
use std::pin::Pin;

use crate::error::TikiResult;

/// Boxed future returned by plugin methods, so plugins can be stored as trait objects.
pub type PluginFuture<'call, T> = Pin<Box<dyn Future<Output = TikiResult<T>> + Send + 'call>>;

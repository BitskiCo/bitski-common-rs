// https://github.com/googleapis/googleapis/blob/master/google/rpc/code.proto
//
// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// https://github.com/hyperium/tonic/blob/master/tonic/src/status.rs
//
// Copyright (c) 2020 Lucio Franco
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Bitski errors.

use std::fmt;

#[cfg(feature = "actix-web")]
use actix_web::ResponseError;

/// Information about an error.
#[derive(Debug, Default)]
pub struct Info {
    /// A message describing the error.
    message: Option<String>,

    /// The lower-level source of this error, if any.
    source: Option<anyhow::Error>,

    /// The [`tonic::Status`] source of this error, if any.
    #[cfg(feature = "tonic")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
    grpc_status: Option<tonic::Status>,

    /// A custom [`http::StatusCode`] for this error.
    #[cfg(feature = "actix-web")]
    #[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
    http_status_code: Option<http::StatusCode>,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(message) = self.message.as_ref() {
            f.write_str(message)
        } else {
            f.write_str("error")
        }
    }
}

/// Common errors.
///
/// These variants match the [`tonic::Status`] variants and [gRPC status codes].
///
/// [gRPC status codes]: https://github.com/grpc/grpc/blob/master/doc/statuscodes.md#status-codes-and-their-use-in-grpc
#[derive(Debug)]
pub enum Error {
    /// The operation was cancelled.
    Cancelled(Info),

    /// Unknown error.
    Unknown(Info),

    /// Client specified an invalid argument.
    InvalidArgument(Info),

    /// Deadline expired before operation could complete.
    DeadlineExceeded(Info),

    /// Some requested entity was not found.
    NotFound(Info),

    /// Some entity that we attempted to create already exists.
    AlreadyExists(Info),

    /// The caller does not have permission to execute the specified operation.
    PermissionDenied(Info),

    /// Some resource has been exhausted.
    ResourceExhausted(Info),

    /// The system is not in a state required for the operation's execution.
    FailedPrecondition(Info),

    /// The operation was aborted.
    Aborted(Info),

    /// Operation was attempted past the valid range.
    OutOfRange(Info),

    /// Operation is not implemented or not supported.
    Unimplemented(Info),

    /// Internal error.
    Internal(Info),

    /// The service is currently unavailable.
    Unavailable(Info),

    /// Unrecoverable data loss or corruption.
    DataLoss(Info),

    /// The request does not have valid authentication credentials
    Unauthenticated(Info),
}

impl Error {
    /// Gets a reference to the error [`Info`].
    pub fn info(&self) -> &Info {
        match self {
            Error::Cancelled(info) => info,
            Error::Unknown(info) => info,
            Error::InvalidArgument(info) => info,
            Error::DeadlineExceeded(info) => info,
            Error::NotFound(info) => info,
            Error::AlreadyExists(info) => info,
            Error::PermissionDenied(info) => info,
            Error::ResourceExhausted(info) => info,
            Error::FailedPrecondition(info) => info,
            Error::Aborted(info) => info,
            Error::OutOfRange(info) => info,
            Error::Unimplemented(info) => info,
            Error::Internal(info) => info,
            Error::Unavailable(info) => info,
            Error::DataLoss(info) => info,
            Error::Unauthenticated(info) => info,
        }
    }

    /// Gets a mutable reference to the error [`Info`].
    pub fn info_mut(&mut self) -> &mut Info {
        match self {
            Error::Cancelled(info) => info,
            Error::Unknown(info) => info,
            Error::InvalidArgument(info) => info,
            Error::DeadlineExceeded(info) => info,
            Error::NotFound(info) => info,
            Error::AlreadyExists(info) => info,
            Error::PermissionDenied(info) => info,
            Error::ResourceExhausted(info) => info,
            Error::FailedPrecondition(info) => info,
            Error::Aborted(info) => info,
            Error::OutOfRange(info) => info,
            Error::Unimplemented(info) => info,
            Error::Internal(info) => info,
            Error::Unavailable(info) => info,
            Error::DataLoss(info) => info,
            Error::Unauthenticated(info) => info,
        }
    }

    /// Sets the `message` for this error.
    pub fn with_message<D: Into<String>>(mut self, message: D) -> Self {
        self.info_mut().message = Some(message.into());
        self
    }

    /// Sets the `source` for this error.
    pub fn with_source<E: Into<anyhow::Error>>(mut self, source: E) -> Self {
        self.info_mut().source = Some(source.into());
        #[cfg(feature = "tonic")]
        {
            self.info_mut().grpc_status = None;
        }
        self
    }

    /// Sets the [`tonic::Status`] as the source for this error.
    #[cfg(feature = "tonic")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
    pub fn with_grpc_status(mut self, status: tonic::Status) -> Self {
        self.info_mut().source = None;
        self.info_mut().grpc_status = Some(status);
        self
    }

    /// Sets a custom [`http::StatusCode`] for this error.
    #[cfg(feature = "actix-web")]
    #[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
    pub fn with_http_status_code(mut self, status_code: http::StatusCode) -> Self {
        self.info_mut().http_status_code = Some(status_code);
        self
    }

    /// The operation was cancelled, typically by the caller.
    ///
    /// HTTP Mapping: 499 Client Closed Request
    pub fn cancelled() -> Self {
        Self::Cancelled(Info::default())
    }

    /// Returns true if this is an `Error::Cancelled` error.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled(_))
    }

    /// Unknown error.  For example, this error may be returned when
    /// a `Status` value received from another address space belongs to
    /// an error space that is not known in this address space.  Also
    /// errors raised by APIs that do not return enough error information
    /// may be converted to this error.
    ///
    /// HTTP Mapping: 500 Internal Server Error
    pub fn unknown() -> Self {
        Self::Unknown(Info::default())
    }

    /// Returns true if this is an `Error::Unknown` error.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    /// The client specified an invalid argument.  Note that this differs
    /// from `FAILED_PRECONDITION`.  `INVALID_ARGUMENT` indicates arguments
    /// that are problematic regardless of the state of the system
    /// (e.g., a malformed file name).
    ///
    /// HTTP Mapping: 400 Bad Request
    pub fn invalid_argument() -> Self {
        Self::InvalidArgument(Info::default())
    }

    /// Returns true if this is an `Error::InvalidArgument` error.
    pub fn is_invalid_argument(&self) -> bool {
        matches!(self, Self::InvalidArgument(_))
    }

    /// The deadline expired before the operation could complete. For operations
    /// that change the state of the system, this error may be returned
    /// even if the operation has completed successfully.  For example, a
    /// successful response from a server could have been delayed long
    /// enough for the deadline to expire.
    ///
    /// HTTP Mapping: 504 Gateway Timeout
    pub fn deadline_exceeded() -> Self {
        Self::DeadlineExceeded(Info::default())
    }

    /// Returns true if this is an `Error::DeadlineExceeded` error.
    pub fn is_deadline_exceeded(&self) -> bool {
        matches!(self, Self::DeadlineExceeded(_))
    }

    /// Some requested entity (e.g., file or directory) was not found.
    ///
    /// Note to server developers: if a request is denied for an entire class
    /// of users, such as gradual feature rollout or undocumented whitelist,
    /// `NOT_FOUND` may be used. If a request is denied for some users within
    /// a class of users, such as user-based access control, `PERMISSION_DENIED`
    /// must be used.
    ///
    /// HTTP Mapping: 404 Not Found
    pub fn not_found() -> Self {
        Self::NotFound(Info::default())
    }

    /// Returns true if this is an `Error::NotFound` error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// The entity that a client attempted to create (e.g., file or directory)
    /// already exists.
    ///
    /// HTTP Mapping: 409 Conflict
    pub fn already_exists() -> Self {
        Self::AlreadyExists(Info::default())
    }

    /// Returns true if this is an `Error::AlreadyExists` error.
    pub fn is_already_exists(&self) -> bool {
        matches!(self, Self::AlreadyExists(_))
    }

    /// The caller does not have permission to execute the specified
    /// operation. `PERMISSION_DENIED` must not be used for rejections
    /// caused by exhausting some resource (use `RESOURCE_EXHAUSTED`
    /// instead for those errors). `PERMISSION_DENIED` must not be
    /// used if the caller can not be identified (use `UNAUTHENTICATED`
    /// instead for those errors). This error code does not imply the
    /// request is valid or the requested entity exists or satisfies
    /// other pre-conditions.
    ///
    /// HTTP Mapping: 403 Forbidden
    pub fn permission_denied() -> Self {
        Self::PermissionDenied(Info::default())
    }

    /// Returns true if this is an `Error::PermissionDenied` error.
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied(_))
    }

    /// Some resource has been exhausted, perhaps a per-user quota, or
    /// perhaps the entire file system is out of space.
    ///
    /// HTTP Mapping: 429 Too Many Requests
    pub fn resource_exhausted() -> Self {
        Self::ResourceExhausted(Info::default())
    }

    /// Returns true if this is an `Error::ResourceExhausted` error.
    pub fn is_resource_exhausted(&self) -> bool {
        matches!(self, Self::ResourceExhausted(_))
    }

    /// The operation was rejected because the system is not in a state
    /// required for the operation's execution.  For example, the directory
    /// to be deleted is non-empty, an rmdir operation is applied to
    /// a non-directory, etc.
    ///
    /// Service implementors can use the following guidelines to decide
    /// between `FAILED_PRECONDITION`, `ABORTED`, and `UNAVAILABLE`:
    ///  (a) Use `UNAVAILABLE` if the client can retry just the failing call.
    ///  (b) Use `ABORTED` if the client should retry at a higher level
    ///      (e.g., when a client-specified test-and-set fails, indicating the
    ///      client should restart a read-modify-write sequence).
    ///  (c) Use `FAILED_PRECONDITION` if the client should not retry until
    ///      the system state has been explicitly fixed.  E.g., if an "rmdir"
    ///      fails because the directory is non-empty, `FAILED_PRECONDITION`
    ///      should be returned since the client should not retry unless
    ///      the files are deleted from the directory.
    ///
    /// HTTP Mapping: 400 Bad Request
    pub fn failed_precondition() -> Self {
        Self::FailedPrecondition(Info::default())
    }

    /// Returns true if this is an `Error::FailedPrecondition` error.
    pub fn is_failed_precondition(&self) -> bool {
        matches!(self, Self::FailedPrecondition(_))
    }

    /// The operation was aborted, typically due to a concurrency issue such as
    /// a sequencer check failure or transaction abort.
    ///
    /// See the guidelines above for deciding between `FAILED_PRECONDITION`,
    /// `ABORTED`, and `UNAVAILABLE`.
    ///
    /// HTTP Mapping: 409 Conflict
    pub fn aborted() -> Self {
        Self::Aborted(Info::default())
    }

    /// Returns true if this is an `Error::Aborted` error.
    pub fn is_aborted(&self) -> bool {
        matches!(self, Self::Aborted(_))
    }

    /// The operation was attempted past the valid range.  E.g., seeking or
    /// reading past end-of-file.
    ///
    /// Unlike `INVALID_ARGUMENT`, this error indicates a problem that may
    /// be fixed if the system state changes. For example, a 32-bit file
    /// system will generate `INVALID_ARGUMENT` if asked to read at an
    /// offset that is not in the range [0,2^32-1], but it will generate
    /// `OUT_OF_RANGE` if asked to read from an offset past the current
    /// file size.
    ///
    /// There is a fair bit of overlap between `FAILED_PRECONDITION` and
    /// `OUT_OF_RANGE`.  We recommend using `OUT_OF_RANGE` (the more specific
    /// error) when it applies so that callers who are iterating through
    /// a space can easily look for an `OUT_OF_RANGE` error to detect when
    /// they are done.
    ///
    /// HTTP Mapping: 400 Bad Request
    pub fn out_of_range() -> Self {
        Self::OutOfRange(Info::default())
    }

    /// Returns true if this is an `Error::OutOfRange` error.
    pub fn is_out_of_range(&self) -> bool {
        matches!(self, Self::OutOfRange(_))
    }

    /// The operation is not implemented or is not supported/enabled in this
    /// service.
    ///
    /// HTTP Mapping: 501 Not Implemented
    pub fn unimplemented() -> Self {
        Self::Unimplemented(Info::default())
    }

    /// Returns true if this is an `Error::Unimplemented` error.
    pub fn is_unimplemented(&self) -> bool {
        matches!(self, Self::Unimplemented(_))
    }

    /// Internal errors.  This means that some invariants expected by the
    /// underlying system have been broken.  This error code is reserved
    /// for serious errors.
    ///
    /// HTTP Mapping: 500 Internal Server Error
    pub fn internal() -> Self {
        Self::Internal(Info::default())
    }

    /// Returns true if this is an `Error::Internal` error.
    pub fn is_internal(&self) -> bool {
        matches!(self, Self::Internal(_))
    }

    /// The service is currently unavailable.  This is most likely a
    /// transient condition, which can be corrected by retrying with
    /// a backoff. Note that it is not always safe to retry
    /// non-idempotent operations.
    ///
    /// See the guidelines above for deciding between `FAILED_PRECONDITION`,
    /// `ABORTED`, and `UNAVAILABLE`.
    ///
    /// HTTP Mapping: 503 Service Unavailable
    pub fn unavailable() -> Self {
        Self::Unavailable(Info::default())
    }

    /// Returns true if this is an `Error::Unavailable` error.
    pub fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }

    /// Unrecoverable data loss or corruption.
    ///
    /// HTTP Mapping: 500 Internal Server Error
    pub fn data_loss() -> Self {
        Self::DataLoss(Info::default())
    }

    /// Returns true if this is an `Error::DataLoss` error.
    pub fn is_data_loss(&self) -> bool {
        matches!(self, Self::DataLoss(_))
    }

    /// The request does not have valid authentication credentials for the
    /// operation.
    ///
    /// HTTP Mapping: 401 Unauthorized
    pub fn unauthenticated() -> Self {
        Self::Unauthenticated(Info::default())
    }

    /// Returns true if this is an `Error::Unauthenticated` error.
    pub fn is_unauthenticated(&self) -> bool {
        matches!(self, Self::Unauthenticated(_))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(message) = self.info().message.as_ref() {
            f.write_str(message)
        } else {
            let repr = match self {
                Error::Cancelled(_) => "Cancelled",
                Error::Unknown(_) => "Unknown",
                Error::InvalidArgument(_) => "Invalid Argument",
                Error::DeadlineExceeded(_) => "Deadline Exceeded",
                Error::NotFound(_) => "Not Found",
                Error::AlreadyExists(_) => "Already Exists",
                Error::PermissionDenied(_) => "Permission Denied",
                Error::ResourceExhausted(_) => "Resource Exhausted",
                Error::FailedPrecondition(_) => "Failed Precondition",
                Error::Aborted(_) => "Aborted",
                Error::OutOfRange(_) => "Out Of Range",
                Error::Unimplemented(_) => "Unimplemented",
                Error::Internal(_) => "Internal",
                Error::Unavailable(_) => "Unavailable",
                Error::DataLoss(_) => "Data Loss",
                Error::Unauthenticated(_) => "Unauthenticated",
            };
            f.write_str(repr)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        #[cfg(feature = "tonic")]
        if let Some(status) = self.info().grpc_status.as_ref() {
            return status.source();
        }

        self.info().source.as_ref().map(AsRef::as_ref)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::internal().with_source(err)
    }
}

impl From<opentelemetry::metrics::MetricsError> for Error {
    fn from(err: opentelemetry::metrics::MetricsError) -> Self {
        Error::internal().with_source(err)
    }
}

impl From<opentelemetry::trace::TraceError> for Error {
    fn from(err: opentelemetry::trace::TraceError) -> Self {
        Error::internal().with_source(err)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::internal().with_source(err)
    }
}

#[cfg(feature = "actix")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix")))]
impl From<actix::MailboxError> for Error {
    fn from(err: actix::MailboxError) -> Self {
        Error::internal().with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::BlockingError> for Error {
    fn from(err: actix_web::error::BlockingError) -> Self {
        Error::internal()
            .with_http_status_code(err.status_code())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::UrlGenerationError> for Error {
    fn from(err: actix_web::error::UrlGenerationError) -> Self {
        Error::internal()
            .with_http_status_code(err.status_code())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::UrlencodedError> for Error {
    fn from(err: actix_web::error::UrlencodedError) -> Self {
        Error::invalid_argument()
            .with_http_status_code(err.status_code())
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::JsonPayloadError> for Error {
    fn from(err: actix_web::error::JsonPayloadError) -> Self {
        Error::invalid_argument()
            .with_http_status_code(err.status_code())
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::PathError> for Error {
    fn from(err: actix_web::error::PathError) -> Self {
        Error::invalid_argument()
            .with_http_status_code(err.status_code())
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::QueryPayloadError> for Error {
    fn from(err: actix_web::error::QueryPayloadError) -> Self {
        Error::invalid_argument()
            .with_http_status_code(err.status_code())
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl From<actix_web::error::ReadlinesError> for Error {
    fn from(err: actix_web::error::ReadlinesError) -> Self {
        Error::invalid_argument()
            .with_http_status_code(err.status_code())
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl ResponseError for Error {
    fn status_code(&self) -> http::StatusCode {
        if let Some(status_code) = self.info().http_status_code {
            return status_code;
        }

        // https://github.com/googleapis/googleapis/blob/master/google/rpc/code.proto
        match self {
            Error::Cancelled(_) => http::StatusCode::from_u16(499).unwrap(),
            Error::Unknown(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidArgument(_) => http::StatusCode::BAD_REQUEST,
            Error::DeadlineExceeded(_) => http::StatusCode::GATEWAY_TIMEOUT,
            Error::NotFound(_) => http::StatusCode::NOT_FOUND,
            Error::AlreadyExists(_) => http::StatusCode::CONFLICT,
            Error::PermissionDenied(_) => http::StatusCode::FORBIDDEN,
            Error::ResourceExhausted(_) => http::StatusCode::TOO_MANY_REQUESTS,
            Error::FailedPrecondition(_) => http::StatusCode::BAD_REQUEST,
            Error::Aborted(_) => http::StatusCode::CONFLICT,
            Error::OutOfRange(_) => http::StatusCode::BAD_REQUEST,
            Error::Unimplemented(_) => http::StatusCode::NOT_IMPLEMENTED,
            Error::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unavailable(_) => http::StatusCode::SERVICE_UNAVAILABLE,
            Error::DataLoss(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            Error::Unauthenticated(_) => http::StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            Error::Cancelled(_)
            | Error::Unknown(_)
            | Error::InvalidArgument(_)
            | Error::PermissionDenied(_)
            | Error::Internal(_)
            | Error::DeadlineExceeded(_)
            | Error::ResourceExhausted(_)
            | Error::Aborted(_)
            | Error::Unimplemented(_)
            | Error::Unavailable(_)
            | Error::DataLoss(_)
            | Error::Unauthenticated(_) => {
                tracing::warn!("{self}");
            }
            _ => {}
        }

        actix_web::HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": {
                "message": self.to_string()
            }
        }))
    }
}

#[cfg(feature = "awc")]
#[cfg_attr(docsrs, doc(cfg(feature = "awc")))]
impl From<awc::error::PayloadError> for Error {
    fn from(err: awc::error::PayloadError) -> Self {
        Error::internal()
            .with_http_status_code(err.status_code())
            .with_source(err)
    }
}

#[cfg(feature = "awc")]
#[cfg_attr(docsrs, doc(cfg(feature = "awc")))]
impl From<awc::error::SendRequestError> for Error {
    fn from(err: awc::error::SendRequestError) -> Self {
        Error::internal().with_message(err.to_string())
    }
}

#[cfg(feature = "awc")]
#[cfg_attr(docsrs, doc(cfg(feature = "awc")))]
impl From<awc::error::JsonPayloadError> for Error {
    fn from(err: awc::error::JsonPayloadError) -> Self {
        Error::internal()
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
impl From<humantime::DurationError> for Error {
    fn from(err: humantime::DurationError) -> Self {
        Error::invalid_argument()
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "humantime")]
#[cfg_attr(docsrs, doc(cfg(feature = "humantime")))]
impl From<humantime::TimestampError> for Error {
    fn from(err: humantime::TimestampError) -> Self {
        Error::invalid_argument()
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "oauth2")]
impl<T: std::error::Error + Send + Sync + 'static>
    From<
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<T>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > for Error
{
    fn from(
        err: oauth2::RequestTokenError<
            oauth2::reqwest::Error<T>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ) -> Self {
        Error::internal()
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        match err.status() {
            Some(status) => match status {
                http::StatusCode::NOT_FOUND => Error::not_found().with_source(err),
                http::StatusCode::TOO_MANY_REQUESTS => Error::resource_exhausted().with_source(err),
                http::StatusCode::SERVICE_UNAVAILABLE | http::StatusCode::GATEWAY_TIMEOUT => {
                    Error::unavailable().with_source(err)
                }
                _ => Error::internal()
                    .with_message(err.to_string())
                    .with_source(err),
            },
            None => Error::internal()
                .with_message(err.to_string())
                .with_source(err),
        }
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => Error::not_found().with_source(err),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Error::already_exists().with_source(err),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => Error::invalid_argument().with_source(err),
            _ => Error::internal().with_source(err),
        }
    }
}

#[cfg(feature = "diesel")]
#[cfg_attr(docsrs, doc(cfg(feature = "diesel")))]
impl From<diesel::result::ConnectionError> for Error {
    fn from(err: diesel::result::ConnectionError) -> Self {
        match err {
            diesel::result::ConnectionError::BadConnection(_)
            | diesel::result::ConnectionError::CouldntSetupConfiguration(_) => {
                Error::unavailable().with_source(err)
            }
            _ => Error::internal().with_source(err),
        }
    }
}

#[cfg(feature = "r2d2")]
#[cfg_attr(docsrs, doc(cfg(feature = "r2d2")))]
impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Self {
        Error::unavailable().with_source(err)
    }
}

#[cfg(feature = "bcrypt")]
#[cfg_attr(docsrs, doc(cfg(feature = "bcrypt")))]
impl From<bcrypt::BcryptError> for Error {
    fn from(err: bcrypt::BcryptError) -> Self {
        Error::internal().with_source(err)
    }
}

#[cfg(feature = "lettre")]
#[cfg_attr(docsrs, doc(cfg(feature = "lettre")))]
impl From<lettre::smtp::error::Error> for Error {
    fn from(err: lettre::smtp::error::Error) -> Self {
        Error::internal()
            .with_message(format!("Error sending email: {}", err))
            .with_source(err)
    }
}

#[cfg(feature = "lettre")]
#[cfg_attr(docsrs, doc(cfg(feature = "lettre")))]
impl From<lettre_email::error::Error> for Error {
    fn from(err: lettre_email::error::Error) -> Self {
        Error::internal()
            .with_message(format!("Error composing email: {}", err))
            .with_source(err)
    }
}

#[cfg(feature = "tonic")]
#[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        let status_code = status.code();

        let info = Info {
            message: Some(status.message().to_owned()),
            source: None,
            #[cfg(feature = "actix-web")]
            http_status_code: None,
            grpc_status: Some(status),
        };

        match status_code {
            tonic::Code::Ok => Error::Unknown(info),
            tonic::Code::Cancelled => Error::Cancelled(info),
            tonic::Code::Unknown => Error::Unknown(info),
            tonic::Code::InvalidArgument => Error::InvalidArgument(info),
            tonic::Code::DeadlineExceeded => Error::DeadlineExceeded(info),
            tonic::Code::NotFound => Error::NotFound(info),
            tonic::Code::AlreadyExists => Error::AlreadyExists(info),
            tonic::Code::PermissionDenied => Error::PermissionDenied(info),
            tonic::Code::ResourceExhausted => Error::ResourceExhausted(info),
            tonic::Code::FailedPrecondition => Error::FailedPrecondition(info),
            tonic::Code::Aborted => Error::Aborted(info),
            tonic::Code::OutOfRange => Error::OutOfRange(info),
            tonic::Code::Unimplemented => Error::Unimplemented(info),
            tonic::Code::Internal => Error::Internal(info),
            tonic::Code::Unavailable => Error::Unavailable(info),
            tonic::Code::DataLoss => Error::DataLoss(info),
            tonic::Code::Unauthenticated => Error::Unauthenticated(info),
        }
    }
}

#[cfg(feature = "tonic")]
#[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
impl From<Error> for tonic::Status {
    fn from(mut err: Error) -> Self {
        if let Some(status) = err.info_mut().grpc_status.take() {
            return status;
        }

        let message = err
            .info_mut()
            .message
            .take()
            .unwrap_or_else(|| "error".into());

        match err {
            Error::Cancelled(_) => tonic::Status::cancelled(message),
            Error::Unknown(_) => tonic::Status::unknown(message),
            Error::InvalidArgument(_) => tonic::Status::invalid_argument(message),
            Error::DeadlineExceeded(_) => tonic::Status::deadline_exceeded(message),
            Error::NotFound(_) => tonic::Status::not_found(message),
            Error::AlreadyExists(_) => tonic::Status::already_exists(message),
            Error::PermissionDenied(_) => tonic::Status::permission_denied(message),
            Error::ResourceExhausted(_) => tonic::Status::resource_exhausted(message),
            Error::FailedPrecondition(_) => tonic::Status::failed_precondition(message),
            Error::Aborted(_) => tonic::Status::aborted(message),
            Error::OutOfRange(_) => tonic::Status::out_of_range(message),
            Error::Unimplemented(_) => tonic::Status::unimplemented(message),
            Error::Internal(_) => tonic::Status::internal(message),
            Error::Unavailable(_) => tonic::Status::unavailable(message),
            Error::DataLoss(_) => tonic::Status::data_loss(message),
            Error::Unauthenticated(_) => tonic::Status::unauthenticated(message),
        }
    }
}

#[cfg(feature = "tonic")]
#[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
impl From<tonic::metadata::errors::InvalidMetadataValue> for Error {
    fn from(err: tonic::metadata::errors::InvalidMetadataValue) -> Self {
        Error::internal().with_source(err)
    }
}

#[cfg(feature = "tonic")]
#[cfg_attr(docsrs, doc(cfg(feature = "tonic")))]
impl From<tonic::metadata::errors::InvalidMetadataValueBytes> for Error {
    fn from(err: tonic::metadata::errors::InvalidMetadataValueBytes) -> Self {
        Error::internal().with_source(err)
    }
}

#[cfg(feature = "validator")]
#[cfg_attr(docsrs, doc(cfg(feature = "validator")))]
impl From<validator::ValidationError> for Error {
    fn from(err: validator::ValidationError) -> Self {
        Error::invalid_argument()
            .with_message(err.message.clone().unwrap_or_else(|| err.code.clone()))
            .with_source(err)
    }
}

#[cfg(feature = "validator")]
#[cfg_attr(docsrs, doc(cfg(feature = "validator")))]
impl From<validator::ValidationErrors> for Error {
    fn from(err: validator::ValidationErrors) -> Self {
        Error::invalid_argument()
            .with_message(err.to_string())
            .with_source(err)
    }
}

#[cfg(all(test, feature = "actix-web"))]
mod test_actix_web {
    use actix_web::ResponseError;

    use super::Error;

    #[test]
    fn test_http_status_code_cancelled() {
        assert_eq!(Error::cancelled().status_code(), 499);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is() {
        assert!(Error::cancelled().is_cancelled());
        assert!(Error::unknown().is_unknown());
        assert!(Error::invalid_argument().is_invalid_argument());
        assert!(Error::deadline_exceeded().is_deadline_exceeded());
        assert!(Error::not_found().is_not_found());
        assert!(Error::already_exists().is_already_exists());
        assert!(Error::permission_denied().is_permission_denied());
        assert!(Error::resource_exhausted().is_resource_exhausted());
        assert!(Error::failed_precondition().is_failed_precondition());
        assert!(Error::aborted().is_aborted());
        assert!(Error::out_of_range().is_out_of_range());
        assert!(Error::unimplemented().is_unimplemented());
        assert!(Error::internal().is_internal());
        assert!(Error::unavailable().is_unavailable());
        assert!(Error::data_loss().is_data_loss());
        assert!(Error::unauthenticated().is_unauthenticated());
    }
}

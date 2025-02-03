use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use axum::http::StatusCode;
use thiserror::Error;
use crate::errors::ProblemResponse;

#[derive(Error, Debug)]
pub enum S3Error {
    #[error("Could not retrieve object from s3 bucket: {0}")]
    Get(#[from] GetObjectError),       
    #[error("Could not upload object into s3 bucket: {0}")]
    Put(#[from] PutObjectError),    
    #[error("Could not delete object from s3 bucket: {0}")]
    Delete(#[from] DeleteObjectError),
}

impl ProblemResponse for S3Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn title(&self) -> &str {
        "Internal Server Error"
    }

    fn detail(&self) -> String {
        self.to_string()
    }
}
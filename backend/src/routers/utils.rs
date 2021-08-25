use crate::config::MAX_PAYLOAD_SIZE;
use actix_web::web;
use flowy_net::{
    errors::{ErrorCode, ServerError},
    response::*,
};
use futures::StreamExt;
use protobuf::{Message, ProtobufResult};

pub async fn parse_from_payload<T: Message>(payload: web::Payload) -> Result<T, ServerError> {
    let bytes = poll_payload(payload).await?;
    parse_from_bytes(&bytes)
}

pub fn parse_from_bytes<T: Message>(bytes: &[u8]) -> Result<T, ServerError> {
    let result: ProtobufResult<T> = Message::parse_from_bytes(&bytes);
    match result {
        Ok(data) => Ok(data),
        Err(e) => Err(e.into()),
    }
}

pub async fn poll_payload(mut payload: web::Payload) -> Result<web::BytesMut, ServerError> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|err| ServerError::internal().context(err))?;

        if (body.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
            return Err(ServerError::new(
                "Payload overflow".to_string(),
                ErrorCode::PayloadOverflow,
            ));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
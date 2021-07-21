use super::*;
use futures::{StreamExt};
use std::str::from_utf8;
use std::sync::{Mutex, MutexGuard};
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::{
    EnvFilter,
};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::fmt::format::Format;
use tracing_test::{traced_test};

async fn get_next_stream_value(stream :&mut ReceiverStream<Result<Bytes, TransmissionError>>) ->String {
    match stream.next().await {
        Some(v)=> {
            let payload = v.unwrap_or_else(|_|Bytes::new());
            let received_payload = from_utf8(&*payload).unwrap_or_else(|_|"");
            String::from(received_payload)
        },
        None => String::from("")
    }
}


#[tokio::test]
async fn should_be_able_to_receive_message_when_one_is_sent() {
    let test_payload = "this is a test";
    let test_correlation_id = "0e88139d-5409-4c91-b652-6bf5e54fd81e";
    let test_client_id = "1";
    let expected_connection_message = CONNECTION_ESTABLISHED;
    let expected_payload = generate_sse_message(test_payload);

    let (_, sse_exchange) = SseExchange::start();

    let mut client_rx = match sse_exchange.connect(test_client_id).await {
        Ok(s)=> s,
        Err(_)=> {
            assert!(false, "unexpected error occur when attempting to connect");
            return;
        }
    };

    let published = sse_exchange.publish(Event::new(
                            test_client_id,
                            test_correlation_id,
                            test_payload)).await;
    assert!(published, "fail to publish");

    assert_eq!(get_next_stream_value(&mut client_rx).await, expected_connection_message);
    assert_eq!(get_next_stream_value(&mut client_rx).await, expected_payload);

    let client_count =sse_exchange.get_client_count().await.unwrap_or_else(|_|0);
    assert_eq!(client_count, 1);
}

#[tokio::test]
#[traced_test]
async fn should_remove_client_when_client_close_rx() {
    let test_client_id = "1";
    let test_correlation_id = "0e88139d-5409-4c91-b652-6bf5e54fd81e";
    let expected_remove_client_log_entry=format!("error sending to client, removing client");

    let (_, sse_exchange) = SseExchange::start();
    let mut client_rx = match sse_exchange.connect(test_client_id).await {
        Ok(s)=> s,
        Err(_)=> {
            assert!(false, "unexpected error occur when attempting to connect");
            return;
        }
    };

    let client_count =sse_exchange.get_client_count().await.unwrap_or_else(|_|0);
    assert_eq!(client_count, 1);

    client_rx.close();

    sse_exchange.publish(Event::new(
        test_client_id,
        test_correlation_id,
        "dummy")).await;

    let client_count =sse_exchange.get_client_count().await.unwrap_or_else(|_|0);
    assert!(logs_contain(expected_remove_client_log_entry.as_str()));
    assert_eq!(client_count, 0);
}

#[tokio::test]
#[traced_test]
async fn should_not_register_client_when_connection_acknowledgement_cannot_be_sent(){
    let test_client_id = "1";
    let expected_connection_failure_log_entry=
        format!("error sending connection acknowledgement to client");

    let (_, sse_exchange) = SseExchange::start();
    let mut client_rx = match sse_exchange.connect(test_client_id).await {
        Ok(s)=> s,
        Err(_)=> {
            assert!(false, "unexpected error occur when attempting to connect");
            return;
        }
    };
    client_rx.close();

    sse_exchange.wait().await;

    let client_count =sse_exchange.get_client_count().await.unwrap_or_else(|_|0);
    assert!(logs_contain(expected_connection_failure_log_entry.as_str()));
    assert_eq!(client_count, 0);
}
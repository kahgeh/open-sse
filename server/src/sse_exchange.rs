use std::collections::HashMap;
use derive_more::{Display, Error};
use actix_web::web::{Bytes};
use tokio::sync::{oneshot, mpsc::{channel, Receiver, Sender}};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info,error};

use crate::sse_exchange::Command::{Connect, Shutdown, Publish, GetClientCount, Wait};
use crate::sse_exchange::SseExchangeError::{FailToEstablishConnection, FailToSendClientCountQuery, FailToGetClientCount, FailToStartWait, FailToWait, FailToStop};

const CONNECTION_ESTABLISHED: &str = "data: connection established\n\n";

#[cfg(test)]
#[path = "sse_exchange_tests.rs"]
mod sse_exchange_tests;

#[derive(Debug, Display, Error)]
pub enum TransmissionError {}

#[derive(Debug, Display, Error)]
pub enum SseExchangeError {
    FailToEstablishConnection,
    FailToSendClientCountQuery,
    FailToGetClientCount(oneshot::error::RecvError),
    FailToStartWait,
    FailToWait(oneshot::error::RecvError),
    FailToStop,
}

pub struct Client {
    id: String,
    sender: Sender<Result<Bytes, TransmissionError>>,
}

impl Client {
    pub fn new(id: &str)->(Client, Receiver<Result<Bytes, TransmissionError>>){
        let (tx, rx) = channel::<Result<Bytes, TransmissionError>>(512);
        (Client {
            id:id.to_string(),
            sender: tx }, rx)
    }
}

pub struct SseExchange {
    tx: Sender<Command>,
}

pub struct Event {
    client_id: String,
    correlation_id: String,
    payload: String,
}

impl Event {
    pub fn new(client_id :&str, correlation_id: &str, payload:&str) -> Event {
        Event {
            client_id: client_id.to_string(),
            payload: payload.to_string(),
            correlation_id: correlation_id.to_string(),
        }
    }
}

pub struct ClientCountQuery {
    tx: tokio::sync::oneshot::Sender<u32>,
}

impl ClientCountQuery {
    pub fn new () -> (oneshot::Receiver<u32>, ClientCountQuery){
        let ( tx, rx) = oneshot::channel::<u32>();
        (rx,ClientCountQuery{tx})
    }
}

pub struct Noop {
    tx: tokio::sync::oneshot::Sender<()>,
}

impl Noop {
    pub fn new () -> (oneshot::Receiver<()>, Noop){
        let ( tx, rx) = oneshot::channel::<()>();
        (rx,Noop{tx})
    }
}

pub enum Command {
    Connect(Client),
    Publish(Event),
    Shutdown,
    GetClientCount(ClientCountQuery),
    Wait(Noop),
}

pub fn generate_sse_message(raw_message:&str)->String{
    format!("data: {}\n\n", raw_message)
}

impl SseExchange {
    pub async fn connect(&self, client_id: &str) -> Result<ReceiverStream<Result<Bytes, TransmissionError>>, SseExchangeError> {
        let (client, rx )=Client::new(client_id);
        if self.tx.clone().send(Connect(client)).await.is_err() {
            error!(client_id=client_id,"error attempting to connect");
            return Err(FailToEstablishConnection)
        }
        info!("returning receiver stream");
        Ok(ReceiverStream::new(rx))
    }

    pub async fn publish(&self, event: Event) -> bool {
        self.tx.clone().send(Publish(event)).await.is_ok()
    }

    pub async fn get_client_count(&self) -> Result<u32,SseExchangeError> {
        let (rx,query) = ClientCountQuery::new();
        if self.tx.clone().send(GetClientCount(query)).await.is_err(){
            return Err(FailToSendClientCountQuery);
        }
        match rx.await {
            Ok(v)=> Ok(v),
            Err(e) =>  Err(FailToGetClientCount(e)),
        }
    }

    pub async fn wait(&self) -> Result<(), SseExchangeError>{
        let (rx,noop) = Noop::new();
        if self.tx.clone().send(Wait(noop)).await.is_err(){
            return Err(FailToStartWait);
        }

        match rx.await {
            Ok(_)=> Ok(()),
            Err(e) =>  Err(FailToWait(e)),
        }
    }

    pub fn start() -> (JoinHandle<tokio::io::Result<()>>, SseExchange) {
        let (tx, mut rx) = channel::<Command>(1024);
        let task=tokio::spawn(async move {
            let mut clients: HashMap<String, Sender<Result<Bytes, TransmissionError>>> = HashMap::new();
            // todo: schedule to close stream and remove clients based on some strategy
            //
            info!("sse exchange started");
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    Command::Connect(client) => {
                        match client.sender.send(Ok(Bytes::from(CONNECTION_ESTABLISHED))).await {
                            Ok(_)=> {
                                clients.insert(client.id.clone(), client.sender);
                                info!(client_id=client.id.as_str(),
                                      "connection acknowledged, client registered");
                            },
                            Err(e)=>{
                                error!(
                                    client_id=client.id.as_str(),
                                    "error sending connection acknowledgement to client {:?}", e);
                            }
                        };
                        info!("acknowledged connect request");
                    },
                    Command::Publish(event) => {
                        match clients.get(&event.client_id) {
                            Some(tx)=> {
                                match tx.send(Ok(Bytes::from(generate_sse_message(&event.payload)))).await {
                                    Ok(_) => {
                                        info!( correlation_id=event.correlation_id.as_str(),
                                               client_id=event.client_id.as_str(),
                                               "sent to client");
                                    },
                                    Err(e) => {
                                        error!(
                                            correlation_id=event.correlation_id.as_str(),
                                            client_id=event.client_id.as_str(),
                                            "error sending to client, removing client [reason:\n {:?}]",e);

                                        clients.remove(&event.client_id);
                                    }
                                }
                            },
                            None => info!(client_id=&event.client_id.as_str(),
                                          correlation_id=&event.correlation_id.as_str(),
                                          "client not registered"),
                        }
                    },
                    Command::GetClientCount(query)=>{
                        if query.tx.send(clients.len() as u32).is_err() {
                            error!("fail to send client count")
                        }
                    }
                    Command::Wait(noop)=>{
                        if noop.tx.send(()).is_err(){
                            error!("fail to send noop completion");
                        }
                    }
                    Command::Shutdown => {
                        info!("stop receiving events");
                        rx.close();
                    }
                }
            }
            Ok(())
        });
        (task,SseExchange{
            tx
        })
    }

    pub async fn stop(&self) -> Result<(), SseExchangeError> {
        info!("shutting down");
        match self.tx.clone().send(Shutdown).await {
            Ok(_)=> Ok(()),
            Err(_)=> {
                error!("error shutting down");
                Err(FailToStop)
            },
        }
    }
}




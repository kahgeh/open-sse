use redis::{Client};
use tracing::error;
use crate::application::StartUpError::FailToGetRedisClient;
use crate::application::StartUpError;

pub fn open_redis (connection_string: &str) -> Result<Client,StartUpError>{
    match redis::Client::open(connection_string){
        Ok(client)=>Ok(client),
        Err(e)=> {
            error!("Fail to create a client to {} ( error - {:?} )", connection_string,e);
            Err(FailToGetRedisClient(e))
        }
    }
}

pub fn test_redis_connection(connection_string: &str)->bool{
    return match open_redis(connection_string) {
        Err(_) => false,
        Ok(client)=>{
            match client.get_connection(){
                Ok(con)=>{
                    let mut con = con;
                    redis::cmd("PING")
                        .execute(&mut con);
                    true
                }
                Err(e)=>{
                    error!("fail to get redis connection because {:?}", e);
                    false
                }
            }
        }
    }
}
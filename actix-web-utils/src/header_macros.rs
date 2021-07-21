#[macro_export]
macro_rules! get_header{
    ($req:expr, $header_name:expr)=>{
        match $req.headers().get($header_name) {
            Some(hv) => match hv.to_str() {
                Ok(v)=>v,
                Err(e)=> {
                    warn!("header value for {} must be made up off ascii characters", $header_name);
                    return HttpResponse::BadRequest().finish();
                },
            },
            None=> return HttpResponse::BadRequest().finish(),
        }
    }
}
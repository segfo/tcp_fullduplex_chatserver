mod server_impl;
mod parser;
// サーバの実装を薄くラップしておく。
// 今のところ特に意味はない。
pub struct Server{
    server:server_impl::Server
}
impl Server{
    pub fn new(listener: std::net::TcpListener)->Self{
        Server{
            server:server_impl::Server::new(listener)
        }
    }
    pub fn serve(&self){
        self.server.serve()
    }
}
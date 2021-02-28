use anyhow::Result;
use futures::future::Future;
use std::fmt;

/// <https://github.com/actix/actix-web/issues/638>

pub struct ServerThread {
    pub join: Option<std::thread::JoinHandle<Result<()>>>,
    pub actix_addr: actix_web::actix::Addr<actix_net::server::Server>,
}

impl fmt::Debug for ServerThread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServerThread: join: {:?}", self.join)
    }
}

impl ServerThread {
    pub fn new<E, H, F>(
        f: impl FnOnce() -> std::result::Result<actix_web::server::HttpServer<H, F>, E> + Send + 'static,
    ) -> std::result::Result<ServerThread, E>
    where
        E: std::error::Error + Send + Sync + 'static,
        H: actix_web::server::IntoHttpHandler + 'static,
        F: Fn() -> H + Send + Clone + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();

        let join = std::thread::spawn(move || {
            let sys = actix::System::new("embedded-http-server");
            match f() {
                Ok(app) => {
                    tx.send(Ok(app.system_exit().start()))?;
                    sys.run();
                }
                Err(e) => {
                    tx.send(Err(e))?;
                }
            }

            Ok(())
        });

        let addr = rx.recv().unwrap()?;

        Ok(ServerThread {
            join: Some(join),
            actix_addr: addr,
        })
    }

    fn stop(&self) {
        self.actix_addr
            .send(actix_web::server::StopServer { graceful: true })
            .wait()
            .unwrap()
            .unwrap();
    }
}

impl Drop for ServerThread {
    fn drop(&mut self) {
        self.stop();
        self.join.take().map(|j| j.join().unwrap());
    }
}

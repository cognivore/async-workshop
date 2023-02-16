use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use hyperpipe::HyperPipe;
use std::{
    path::{Path, PathBuf},
    task::Waker,
    thread::spawn,
};

use std::sync::{Arc, Mutex};
use std::thread;

use inotify::*;

pub struct Never;

pub struct AsyncHyperPipe {
    inner: HyperPipe,
    // path: PathBuf,
    shared_waker: Arc<Mutex<Option<Waker>>>,
}

impl Future for AsyncHyperPipe {
    type Output = Vec<u8>;
    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.get_mut();
        //dbg!("Polling {:?}", ctx);
        match this.inner.pull() {
            Some(buf) => Poll::Ready(buf),
            None => {
                dbg!("No data, waiting");
                // Put the new waker into shared_waker
                *this.shared_waker.lock().unwrap() = Some(ctx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl Future for Never {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        dbg!(Poll::Pending)
    }
}

pub fn manifest_path(root: PathBuf) -> PathBuf {
    root.join("manifest")
}

// Create a thread that takes a future and wakes it up when inotifier notifies us that a file was
// updated
pub fn metadata_notifier(root_path: PathBuf, w: Arc<Mutex<Option<Waker>>>) -> () {
    // Spawn a thread that will wait for a file to be updated
    spawn(move || {
        let mut inotifier = Inotify::init().unwrap();
        inotifier
            .add_watch(manifest_path(root_path.clone()), WatchMask::MOVE)
            .unwrap();
        let mut ibu = [0; 1024];
        loop {
            inotifier.read_events_blocking(&mut ibu).unwrap();
            match w.lock().unwrap().take() {
                Some(waker) => {
                    dbg!("Waking {:?}", waker.clone());
                    waker.wake_by_ref()
                }
                None => panic!("No waker"),
            }
        }
    });
    ()
}

fn main() {
    println!("Hello, world?");
    // smol::block_on(never());
    // smol::block_on(async { Never.await });
    let pipe_path = Path::new("buffer-dir");

    let shared_waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));

    // Note that we can run notifier before the pipe is created, it won't twy to wake unless the
    // file is moved! Which should only happen *after* the pipe already ran.
    metadata_notifier(pipe_path.to_path_buf(), shared_waker.clone());

    let p1 = HyperPipe::new(pipe_path, 2000).unwrap();
    let mut ap1 = AsyncHyperPipe {
        inner: p1,
        shared_waker: shared_waker.clone(),
    };
    let v1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 24];
    // Spawn `ap1.inner.push(v1.clone()).unwrap();` in a thread:
    spawn(move || {
        ap1.inner.push(v1.clone()).unwrap();
    });

    let p2 = HyperPipe::new(pipe_path, 2000).unwrap();
    let ap2 = AsyncHyperPipe {
        inner: p2,
        shared_waker: shared_waker.clone(),
    };

    dbg!("Let's go!");
    smol::block_on(async {
        let y = ap2.await;
        dbg!(y);
    });

    /*
    let p1 = HyperPipe::new(pipe_path).unwrap();
    let mut ap1 = AsyncHyperPipe { inner: p1 };
    let v1 = vec![1, 2, 3, 4, 5, 6];
    ap1.inner.push(v1.clone()).unwrap();

    let p2 = HyperPipe::new(pipe_path).unwrap();
    let ap2 = AsyncHyperPipe { inner: p2 };

    dbg!("Let's go!");
    smol::block_on(async {
        let y = ap2.await;
        dbg!(y);
    });
    */

    /*
    let mut p2 = HyperPipe::new(pipe_path).unwrap();
    let v2 = p2.pull().unwrap();
    // let v2: Vec<u8> = vec![];
    assert_eq!(v1, v2);
    */
}

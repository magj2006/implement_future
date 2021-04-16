use std::{cell::RefCell, thread_local};

thread_local! {static NOTIFY: RefCell<bool> = RefCell::new(true)}

struct Context<'a> {
    waker: &'a Waker,
}

impl<'a> Context<'a> {
    fn from_waker(waker: &'a Waker) -> Self {
        Context { waker }
    }

    fn waker(&self) -> &'a Waker {
        &self.waker
    }
}
struct Waker;

impl Waker {
    pub fn wake(&self) {
        NOTIFY.with(|is_ready| *is_ready.borrow_mut() = true)
    }
}

enum Poll<T> {
    Pending,
    Ready(T),
}

trait Future {
    type Output;

    fn poll(self: &mut Self, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

#[derive(Default)]
struct MyFuture {
    progress: u32,
}

impl Future for MyFuture {
    type Output = u32;
    fn poll(self: &mut Self, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Checking whether ready to poll or not");
        match self.progress {
            100 => Poll::Ready(100),
            _ => {
                self.progress += 20;
                cx.waker().wake();
                Poll::Pending
            }
        }
    }
}

fn run<F>(mut future: F) -> F::Output
where
    F: Future,
{
    NOTIFY.with(|is_ready| loop {
        if *is_ready.borrow() {
            *is_ready.borrow_mut() = false;
            let mut cx = Context::from_waker(&Waker);
            if let Poll::Ready(value) = future.poll(&mut cx) {
                println!("Finally. future is complete");
                return value;
            }
        }
    })
}

fn main() {
    let my_future = MyFuture::default();

    println!("{} future is complete", run(my_future));
}

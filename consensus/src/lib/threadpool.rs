use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::thread;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send +'static>;

struct Worker {
    id:usize,
    thread_handler:JoinHandle<()>
}

impl Worker {
    fn new(_id:usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread_instance = thread::spawn( move||{
            loop {
                let job_result  = receiver.lock().unwrap().recv();

                if job_result.is_ok() {
                    let job = job_result.unwrap();
                    job.call_box();
                }

            }
        });
        let worker = Worker {
            id:_id,
            thread_handler:thread_instance
        };
        return worker;
    }
}

pub struct ThreadPool {
    size:usize,
    workers:Vec<Worker>,
    sender: mpsc::Sender<Job>
}


impl ThreadPool {
    pub fn new(_size:usize) ->ThreadPool{
        let mut workerVec = Vec::with_capacity(_size);

        let (_sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0.._size {

            workerVec.push(Worker::new(id, Arc::clone(&receiver)));
        }

        return ThreadPool {
            size:_size,
            workers:workerVec,
            sender:_sender
        };
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send +'static {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

}
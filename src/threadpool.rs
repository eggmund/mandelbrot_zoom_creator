use rug::{Complex, Float};

use std::boxed::Box;
use std::sync::{mpsc, mpsc::RecvError, Arc, Mutex};
use std::thread;

use crate::mandelbrot;

pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,

    job_sender: mpsc::Sender<Job>,
    output_receiver: mpsc::Receiver<JobResult>,
}

impl ThreadPool {
    pub fn new(worker_num: usize) -> ThreadPool {
        let (job_sender, job_receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();

        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let mut t = ThreadPool {
            workers: Vec::with_capacity(worker_num),
            job_sender,
            output_receiver,
        };

        println!("Worker num: {}", worker_num);

        for _ in 0..worker_num {
            let job_receiver = Arc::clone(&job_receiver);
            let output_sender = output_sender.clone();

            t.workers.push(thread::spawn(move || {
                loop {
                    let job = match job_receiver.lock().unwrap().recv() {
                        Ok(job) => job,
                        Err(_) => break,
                    };

                    let real_increase: &Float = &*job.z_real_increase;

                    let mut ratios: Vec<Option<f32>> = Vec::with_capacity(job.image_width as usize);

                    let mut z0 = job.z_start;

                    for _ in 0..job.image_width {
                        ratios.push(mandelbrot::escape_and_get_ratio(job.max_iter, &z0));

                        *z0.mut_real() += real_increase;
                    }
                    
                    output_sender
                        .send(JobResult {
                            row: job.row,
                            ratios,
                        })
                        .unwrap();
                }
            }));
        }

        t
    }

    #[inline]
    pub fn send_job(&self, job: Job) {
        self.job_sender.send(job).unwrap();
    }

    #[inline]
    pub fn get_job_result(&self) -> Result<JobResult, RecvError> {
        self.output_receiver.recv()
    }
}

pub struct Job {
    pub row: u32,
    pub image_width: u32,
    pub z_start: Box<Complex>, // Put it on heap
    pub z_real_increase: Arc<Box<Float>>,
    pub max_iter: usize,
}

pub struct JobResult {
    pub row: u32,
    pub ratios: Vec<Option<f32>>,
}

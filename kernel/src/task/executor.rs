use core::task::{Context, Poll, Waker};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use conquer_once::{spin::OnceCell};
use crossbeam::queue::ArrayQueue;
use x86_64::instructions::interrupts::{self, enable_and_hlt};

use crate::task::{Task, TaskId};

const MAX_NUM_TASKS: usize = 100;
const SPAWN_BUFFER_LENGTH: usize = 50;

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue
        }))
    }
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

static INBOX: OnceCell<ArrayQueue<Task>> = OnceCell::uninit();

pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
    let inbox = INBOX.get().expect("Inbox not initialized");

    if inbox.push(Task::new(f)).is_err() {
        panic!("spawn inbox full!")
    }
}

impl Executor {
    pub fn new() -> Self {
        INBOX.init_once(|| ArrayQueue::new(SPAWN_BUFFER_LENGTH) );

        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(MAX_NUM_TASKS)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already exists in tasks")
        }

        self.task_queue.push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn drain_inbox(&mut self) {
        let inbox = INBOX.get().expect("executor not initialized");

        while let Some(task) = inbox.pop() {
            self.spawn(task);
        }
    }

    fn run_ready_tasks(&mut self) {
        self.drain_inbox();

        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut( &task_id) {
                Some(task) => task,
                // that shit gone
                None => { continue; }
            };

            let waker = self.waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, self.task_queue.clone()));

            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                },
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.task_queue.is_empty() && INBOX.get().expect("executor not initialized").is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

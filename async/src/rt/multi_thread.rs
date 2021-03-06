use std::thread;
use std::sync::Arc;
use std::io::Result;
use std::future::Future;
use std::time::Duration;
use std::thread::Builder;
use std::cell::UnsafeCell;
use std::task::{Waker, Context, Poll};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crossbeam_channel::unbounded;
use parking_lot::{Mutex, Condvar};
use futures::{future::{FutureExt, BoxFuture}, task::{ArcWake, waker_ref}, TryFuture};

use crate::{AsyncTask,
            lock::steal_deque::{Sender as StealSent, Receiver as StealRecv, steal_deque}};
use super::{TaskId, AsyncRuntime, AsyncTaskTimer, AsyncWaitTimeout, AsyncWait, AsyncWaitAny, AsyncMap, alloc_rt_uid};

/*
* 线程唯一id
*/
thread_local! {
    static THREAD_LOCAL_ID: UnsafeCell<usize> = UnsafeCell::new(0);
    static THREAD_LOCAL_STATUS: AtomicBool = AtomicBool::new(false);
}

/*
* 多线程任务
*/
pub struct MultiTask<O: Default + 'static> {
    uid:    TaskId,                                     //任务唯一id
    future: UnsafeCell<Option<BoxFuture<'static, O>>>,  //异步任务
    queue:  Arc<MultiTasks<O>>,                         //任务唤醒队列
}

unsafe impl<O: Default + 'static> Send for MultiTask<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTask<O> {}

impl<O: Default + 'static> ArcWake for MultiTask<O> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let origin_thread = arc_self.queue.id;
        if let Err(_) = THREAD_LOCAL_ID.try_with(move |id| {
            unsafe {
                if (*id.get()) == origin_thread {
                    //唤醒线程即任务所在运行时的所在工作者线程，则不需要通知
                    if let Some(last_task) = arc_self.queue.try_push_back(arc_self.clone()) {
                        //尝试推入当前运行时线程的发送缓冲区尾失败，则推入当前运行时线程的接收队列尾
                        arc_self.queue.push_recv_back(last_task);
                    }
                } else {
                    //唤醒线程不是任务所在工作者线程，则可能需要通知任务所在的工作者线程
                    let _ = arc_self.queue.push_back_notify(arc_self.clone());
                }
            }
        }) {
            //无效的线程唯一id，则默认不需要通知
            let _ = arc_self.queue.push_back(arc_self.clone());
        }
    }
}

impl<O: Default + 'static> AsyncTask for MultiTask<O> {
    type Out = O;

    fn get_inner(&self) -> Option<BoxFuture<'static, Self::Out>> {
        unsafe { (*self.future.get()).take() }
    }

    fn set_inner(&self, inner: Option<BoxFuture<'static, Self::Out>>) {
        unsafe { *self.future.get() = inner; }
    }
}

impl<O: Default + 'static> MultiTask<O> {
    //构建多线程任务
    pub fn new(uid: TaskId, queue: Arc<MultiTasks<O>>, future: Option<BoxFuture<'static, O>>) -> MultiTask<O> {
        MultiTask {
            uid,
            future: UnsafeCell::new(future),
            queue,
        }
    }

    //检查是否允许唤醒
    pub fn is_enable_wakeup(&self) -> bool {
        self.uid.0.load(Ordering::Relaxed) > 0
    }
}

/*
* 多线程任务队列
*/
pub struct MultiTasks<O: Default + 'static> {
    id:             usize,                          //绑定的线程唯一id
    consumer:       StealRecv<Arc<MultiTask<O>>>,   //任务消费者
    producer:       StealSent<Arc<MultiTask<O>>>,   //任务生产者
    worker_waker:   Arc<(Mutex<bool>, Condvar)>,    //工作者唤醒器
    recv_counter:   Arc<AtomicUsize>,               //接收队列计数器
}

unsafe impl<O: Default + 'static> Send for MultiTasks<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTasks<O> {}

impl<O: Default + 'static> Clone for MultiTasks<O> {
    fn clone(&self) -> Self {
        MultiTasks {
            id: self.id,
            consumer: self.consumer.clone(),
            producer: self.producer.clone(),
            worker_waker: self.worker_waker.clone(),
            recv_counter: self.recv_counter.clone(),
        }
    }
}

impl<O: Default + 'static> MultiTasks<O> {
    //获取任务数量，不精确
    #[inline]
    pub fn len(&self) -> usize {
        self.producer.len() + self.consumer.len()
    }

    //尝试向多线程任务队列尾推入指定的任务
    pub fn try_push_back(&self, task: Arc<MultiTask<O>>) -> Option<Arc<MultiTask<O>>> {
        if let Some(task) = self.producer.try_send(1, task) {
            //尝试推入指定的任务失败
            return Some(task);
        }

        None
    }

    //向多线程任务队列尾推入指定的任务
    pub fn push_back(&self, task: Arc<MultiTask<O>>) -> Result<()> {
        self.producer.send(task);
        Ok(())
    }

    //尝试向多线程任务队列尾推入指定的任务，并根据需要通知控制者唤醒休眠的工作者，成功返回空
    pub fn try_push_back_notify(&self, task: Arc<MultiTask<O>>) -> Option<Arc<MultiTask<O>>> {
        if let Some(task) = self.producer.try_send(1, task) {
            //尝试推入指定的任务失败
            return Some(task);
        }

        let worker_waker = self.worker_waker.clone();
        let _ = THREAD_LOCAL_STATUS.try_with(move |status| {
            if !status.swap(true, Ordering::SeqCst) {
                //需要唤醒工作者
                let (lock, cvar) = &**&worker_waker;
                let mut status = lock.lock();
                *status = true;
                cvar.notify_one();
            }
        });

        None
    }

    //向多线程任务队列尾推入指定的任务，并根据需要通知控制者唤醒休眠的工作者
    pub fn push_back_notify(&self, task: Arc<MultiTask<O>>) -> Result<()> {
        self.producer.send(task);

        let worker_waker = self.worker_waker.clone();
        let _ = THREAD_LOCAL_STATUS.try_with(move |status| {
            if !status.swap(true, Ordering::SeqCst) {
                //需要唤醒工作者
                let (lock, cvar) = &**&worker_waker;
                let mut status = lock.lock();
                *status = true;
                cvar.notify_one();
            }
        });

        Ok(())
    }

    //向多线程任务接收队列尾推入指定的任务
    pub fn push_recv_back(&self, task: Arc<MultiTask<O>>) {
        self.consumer.append(task, &self.recv_counter);
    }
}

/*
* 异步多线程任务运行时，支持运行时线程间任务窃取
*/
pub struct MultiTaskRuntime<O: Default + 'static>(Arc<(
    usize,                                          //运行时唯一id
    AtomicUsize,                                    //异步任务计数器
    Arc<Vec<Arc<MultiTasks<O>>>>,                   //异步任务队列
    Arc<AtomicUsize>,                               //所有待处理任务数量，只包括所有接收队列的任务数量
    Option<AsyncTaskTimer>,                         //本地定时器
)>);

unsafe impl<O: Default + 'static> Send for MultiTaskRuntime<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTaskRuntime<O> {}

impl<O: Default + 'static> Clone for MultiTaskRuntime<O> {
    fn clone(&self) -> Self {
        MultiTaskRuntime(self.0.clone())
    }
}

/*
* 异步多线程任务运行时同步方法
*/
impl<O: Default + 'static> MultiTaskRuntime<O> {
    //获取当前运行时的唯一id
    pub fn get_id(&self) -> usize {
        (self.0).0
    }

    //获取当前运行时的工作者线程数量
    pub fn worker_size(&self) -> usize {
        (self.0).2.len()
    }

    //获取当前运行时待处理任务数量
    pub fn wait_len(&self) -> usize {
        (self.0).3.load(Ordering::Relaxed)
    }

    //获取当前运行时任务数量，不精确
    pub fn len(&self) -> usize {
        let mut len = 0;
        for tasks in (self.0).2.iter() {
            len += tasks.len();
        }
        len
    }

    //分配异步任务的唯一id
    pub fn alloc(&self) -> TaskId {
        TaskId(Arc::new(AtomicUsize::new(0)))
    }

    //派发一个指定的异步任务到异步多线程运行时
    pub fn spawn<F>(&self, task_id: TaskId, future: F) -> Result<()>
        where F: Future<Output = O> + Send + 'static {
        let queues = &(self.0).2;
        let queues_len = queues.len();

        let _ = THREAD_LOCAL_ID.try_with(move |id| {
            let thread_id = unsafe { *id.get() };
            if (self.0).0 == (thread_id >> 8 & 0xff) {
                //当前派发线程，是当前运行时线程，则派发任务到当前运行时线程的任务队列
                let queue = &queues[(thread_id & 0xff) - 1];
                let task = Arc::new(MultiTask::new(task_id, queue.clone(), Some(Box::new(future).boxed())));

                if let Some(last_task) = queue.try_push_back(task) {
                    //尝试当前队列发送缓冲区尾推送失败，则更换到当前队列的接收队列尾
                    queue.push_recv_back(last_task);
                }
            } else {
                //当前派发线程，不是当前运行时线程，则随机选择派发的任务队列
                let m = queues_len - 1;
                let mut index: usize = (self.0).1.fetch_add(1, Ordering::Relaxed) % (self.0).2.len(); //随机选择一个线程的队列
                let queue = &queues[index];
                let mut task = Arc::new(MultiTask::new(task_id, queue.clone(), Some(Box::new(future).boxed())));

                loop {
                    if let Some(last_task) = queue.try_push_back_notify(task) {
                        //尝试当前队列推送失败，则更换到其它队列
                        task = last_task;
                        index += 1;
                        if let Some(r) = Arc::get_mut(&mut task) {
                            r.queue = queues[m - index % queues_len].clone();
                        }
                        continue;
                    } else {
                        //尝试当前队列推送成功，则立即退出
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    //挂起指定唯一id的异步任务
    pub fn pending<Output>(&self, task_id: &TaskId, waker: Waker) -> Poll<Output> {
        task_id.0.store(Box::into_raw(Box::new(waker)) as usize, Ordering::Relaxed);
        Poll::Pending
    }

    //唤醒执行指定唯一id的异步任务
    pub fn wakeup(&self, task_id: &TaskId) {
        match task_id.0.load(Ordering::Relaxed) {
            0 => panic!("Multi runtime wakeup task failed, reason: task id not exist"),
            ptr => {
                unsafe {
                    let waker = Box::from_raw(ptr as *mut Waker);
                    waker.wake();
                }
            },
        }
    }

    //构建用于派发多个异步任务到指定运行时的映射
    pub fn map<V: Send + 'static>(&self) -> AsyncMap<O, V> {
        let (producor, consumer) = unbounded();

        AsyncMap {
            count: 0,
            futures: Vec::new(),
            producor,
            consumer,
        }
    }
}

/*
* 异步多线程任务运行时异步方法
*/
impl<O: Default + 'static> MultiTaskRuntime<O> {
    //挂起当前多线程运行时的当前任务，等待指定的时间后唤醒当前任务
    pub async fn wait_timeout(&self, timeout: usize) {
        if let Some(timer) = (self.0).4.as_ref() {
            //有本地定时器，则异步等待指定时间
            AsyncWaitTimeout::new(AsyncRuntime::Multi(self.clone()), timer.get_producor(), timeout).await
        } else {
            //没有本地定时器，则同步休眠指定时间
            thread::sleep(Duration::from_millis(timeout as u64));
        }
    }

    //挂起当前多线程运行时的当前任务，并在指定的其它运行时上派发一个指定的异步任务，等待其它运行时上的异步任务完成后，唤醒当前运行时的当前任务，并返回其它运行时上的异步任务的值
    pub async fn wait<R, V, F>(&self, rt: AsyncRuntime<R>, future: F) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static,
              F: Future<Output = Result<V>> + Send + 'static {
        AsyncWait::new(AsyncRuntime::Multi(self.clone()), rt, Some(Box::new(future).boxed())).await
    }

    //挂起当前多线程运行时的当前任务，并在多个其它运行时上执行多个其它任务，其中任意一个任务完成，则唤醒当前运行时的当前任务，并返回这个已完成任务的值，而其它未完成的任务的值将被忽略
    pub async fn wait_any<R, V>(&self, futures: Vec<(AsyncRuntime<R>, BoxFuture<'static, Result<V>>)>) -> Result<V>
        where R: Default + 'static,
              V: Send + 'static  {
        AsyncWaitAny::new(AsyncRuntime::Multi(self.clone()), futures).await
    }
}

/*
* 异步多线程任务池
*/
pub struct MultiTaskPool<O: Default + 'static> {
    runtime:    MultiTaskRuntime<O>,            //异步多线程任务运行时
    timeout:    u64,                            //工作者空闲时最长休眠时间
    builders:   Vec<Builder>,                   //工作者构建器列表
    interval:   Option<u64>,                    //定时器运行间隔时间
    builer:     Option<Builder>,                //定时器构建器
}

unsafe impl<O: Default + 'static> Send for MultiTaskPool<O> {}
unsafe impl<O: Default + 'static> Sync for MultiTaskPool<O> {}

impl<O: Default + 'static> MultiTaskPool<O> {
    //构建指定线程名前缀、线程数量、线程栈大小、线程空闲时最长休眠时间和是否使用本地定时器的多线程任务池
    pub fn new(prefix: String, mut size: usize, stack_size: usize, timeout: u64, interval: Option<u64>) -> Self {
        if size == 0 {
            //如果线程太少，则设置至少1个线程
            size = 1;
        }

        let mut builders = Vec::with_capacity(size);
        for idx in 0..size {
            let builder = Builder::new()
                .name(prefix.to_string() + "-" + idx.to_string().as_str())
                .stack_size(stack_size);
            builders.push(builder);
        }

        //构建多线程任务队列
        let rt_uid = alloc_rt_uid();
        let mut queues = Vec::with_capacity(size);
        let counter = Arc::new(AtomicUsize::new(0));
        for index in 0..size {
            let (producer, consumer) = steal_deque();
            let worker_waker = Arc::new((Mutex::new(false), Condvar::new()));
            let queue = Arc::new(MultiTasks {
                id: (rt_uid << 8) & 0xffff | (index + 1) & 0xff,
                consumer,
                producer,
                worker_waker,
                recv_counter: counter.clone(),
            });
            queues.push(queue);
        }

        //构建本地定时器
        let (timer, builer) = if let Some(interval) = interval {
            let builder = Builder::new()
                .name(prefix.to_string() + "-Timer")
                .stack_size(stack_size);
            (Some(AsyncTaskTimer::new()), Some(builder))
        } else {
            (None, None)
        };

        //构建多线程任务运行时
        let runtime = MultiTaskRuntime(Arc::new((
            rt_uid,
            AtomicUsize::new(0),
            Arc::new(queues),
            counter,
            timer,
        )));

        MultiTaskPool {
            runtime,
            timeout,
            builders,
            interval,
            builer,
        }
    }

    //在启动前获取异步运行时
    pub fn runtime(&self) -> &MultiTaskRuntime<O> {
        &self.runtime
    }

    //启动异步多线程任务池，如果任务有大量或长时间的阻塞则建议允许窃取，否则建议不允许窃取
    pub fn startup(mut self, enable_steal: bool) -> MultiTaskRuntime<O> {
        if let Some(builer) = self.builer.take() {
            //启动本地定时器
            let runtime = self.runtime.clone();
            let interval = self.interval.unwrap();
            builer.spawn(move || {
                timer_loop(runtime, interval);
            });
        }

        //启动工作线程
        for index in 0..self.builders.len() {
            let builder = self.builders.remove(0);
            let runtime = self.runtime.clone();
            let timeout = self.timeout;
            let _ = builder.spawn(move || {
                work_loop(runtime, index, enable_steal, timeout);
            });
        }

        self.runtime
    }
}

//定时器循环
fn timer_loop<O: Default + 'static>(runtime: MultiTaskRuntime<O>, interval: u64) {
    loop {
        //设置新的定时任务，并唤醒已过期的定时任务
        (runtime.0).4.as_ref().unwrap().consume();
        for expired in &(runtime.0).4.as_ref().unwrap().poll() {
            runtime.wakeup(expired);
        }

        //间隔指定时间后继续
        thread::sleep(Duration::from_millis(interval));
    }
}

//线程工作循环
fn work_loop<O: Default + 'static>(runtime: MultiTaskRuntime<O>,
                                   queue_index: usize,
                                   enable_steal: bool,
                                   timeout: u64) {
    //初始化当前线程的线程id和线程活动状态
    let queue = (runtime.0).2.get(queue_index).unwrap();
    let worker_waker = &queue.worker_waker;
    let thread_id = queue.id;
    if let Err(e) = THREAD_LOCAL_ID.try_with(move |id| {
        unsafe { (*id.get()) = thread_id; }
    }) {
        panic!("Multi thread runtime startup failed, thread id: {:?}, reason: {:?}", thread_id, e);
    }
    if let Err(e) = THREAD_LOCAL_STATUS.try_with(move |status| {
        status.store(true, Ordering::Relaxed);
    }) {
        panic!("Multi thread runtime startup failed, thread id: {:?}, reason: {:?}", thread_id, e);
    }

    let counter = &(runtime.0).3;
    loop {
        match queue.consumer.try_recv(counter) {
            None => {
                //当前没有任务
                if enable_steal {
                    //允许窃取任务
                    if try_steal_task(&runtime, queue) {
                        //尝试窃取成功，则继续工作
                        continue;
                    }
                }

                //获取任务失败，则准备休眠
                let _ = THREAD_LOCAL_STATUS.try_with(move |status| {
                    status.store(false, Ordering::SeqCst); //设置线程为非活动
                });
                let (lock, cvar) = &**worker_waker;
                let mut status = lock.lock();
                while !*status {
                    //让当前工作者休眠，等待有任务时被唤醒或超时后自动唤醒
                    let wait = cvar.wait_for(&mut status, Duration::from_millis(timeout));
                     if wait.timed_out() {
                        //超时后自动唤醒，则更新工作者唤醒状态，并退出唤醒状态的检查
                        break;
                    }
                }
                *status = false; //重置工作者唤醒状态，并继续工作
            },
            Some(task) => {
                run_task(&runtime, queue, task);
            },
        }
    }
}

//尝试窃取其它工作者队列的异步任务，返回窃取是否成功
fn try_steal_task<O: Default + 'static>(runtime: &MultiTaskRuntime<O>, queue: &Arc<MultiTasks<O>>) -> bool {
    let limit = runtime.worker_size();
    let mut steal_count = runtime.wait_len() / runtime.worker_size();
    if steal_count >= limit {
        //最多尝试窃取当前工作者线程数减一次
        steal_count = limit - 1;
    }

    let m = limit - 1;
    let ignore_index = (queue.id & 0xff) - 1;
    let mut index = m - ignore_index % limit; //获取起始工作者队列序号，默认从当前工作者队列序号的下一个开始
    let mut idx = index;
    let queues = &(runtime.0).2;
    for _ in 0..steal_count {
        if index == ignore_index {
            //跳过当前工作者队列
            idx += 1;
            index = m - idx % limit;
            continue;
        }

        //窃取工作者队列的发送缓冲区
        if !queues[index].producer.try_is_empty() {
            //快速检查发送缓冲区不为空，则窃取
            if let Some(mut buf) = queues[index].producer.try_take(3) {
                if buf.len() > 0 {
                    // println!("!!!!!!{:?} steal sent buf ok from {:?}, len: {:?}", queue.id, index + 1, buf.len());
                    //再次确认发送缓冲区不为空，则将窃取的任务加入发送缓冲区尾部，并立即结束本次窃取
                    queue.producer.append(&mut buf);
                    return true;
                }
            }
        }

        //窃取工作者队列的接收队列
        if !queues[index].consumer.is_empty_recv() {
            //快速检查接收队列不为空，则窃取
            if let Some(deque) = queues[index].consumer.take() {
                if deque.len() > 0 {
                    // println!("!!!!!!{:?} steal recv deque ok from {:?}, len: {:?}", queue.id, index + 1, deque.len());
                    //再次确认接收队列不为空，则将窃取的任务加入发送缓冲区尾部，并立即结束本次窃取
                    queue.producer.append(&mut deque.into());
                    return true;
                }
            }
        }

        //没有窃取到任务，则继续尝试窃取下一个工作者队列
        idx += 1;
        index = m - idx % limit;
    }

    false
}

//执行异步任务
fn run_task<O: Default + 'static>(runtime: &MultiTaskRuntime<O>, queue: &Arc<MultiTasks<O>>, mut task: Arc<MultiTask<O>>) {
    if task.queue.id != queue.id {
        if let Some(future) = task.get_inner() {
            //当前任务是窃取的任务，则替换此任务的id和队列
            task = Arc::new(MultiTask::new(runtime.alloc(), queue.clone(), Some(future)));
        } else {
            //窃取的任务的内部任务还未恢复，则将此任务放回发送缓冲区尾
            let _ = queue.push_back(task);
            return;
        }
    }

    let waker = waker_ref(&task);
    let mut context = Context::from_waker(&*waker);
    if let Some(mut future) = task.get_inner() {
        if let Poll::Pending = future.as_mut().poll(&mut context) {
            //当前未准备好，则恢复异步任务，以保证异步服务后续访问异步任务和异步任务不被提前释放
            task.set_inner(Some(future));
        }
    }
}

from queue import Queue
from threading import Thread


class SimpleThreadPool:
    """Used for benchmarking multithreaded performance. To avoid benchmarking eg ThreadPoolExecutor itself"""

    class Worker(Thread):
        def __init__(self, tasks: Queue):
            Thread.__init__(self, daemon=True)
            self.tasks = tasks
            self.start()

        def run(self):
            while True:
                task = self.tasks.get()
                try:
                    if task is None:
                        break
                    func, args, kwargs = task
                    func(*args, **kwargs)
                finally:
                    self.tasks.task_done()

    def __init__(self, num_threads: int):
        self.tasks: Queue = Queue()
        self.workers = [SimpleThreadPool.Worker(self.tasks) for _ in range(num_threads)]

    def add_task(self, func, *args, **kwargs):
        self.tasks.put((func, args, kwargs))

    def wait_completion(self):
        self.tasks.join()

    def close(self):
        self.tasks.join()
        for _ in self.workers:
            self.tasks.put(None)
        for worker in self.workers:
            worker.join()

use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};

//TODO : Analyze code to check if semaphores worka as intended
//Right now it seems like we have big pause between initial 4 tasks in semaphore

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a semaphore maximum of 4 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(4));
    let mut tasks: JoinSet<Result<(), JoinError>> = JoinSet::new();

    let contents = std::fs::read_to_string("jobs_Success_909_jobs.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    //allows Us to share the browser across multiple tasks.
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    // Create a channel to send requests to the processing task
    let (tx, rx) = mpsc::channel(100);

    // Spawn a task to do all the processing. Since this is a single
    // task, all processing will be done sequentially.
    tokio::spawn(async move {
        process(rx).await;
    });

    // Iterate over the urls and send them to the processing task
    for url in urls {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();
        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 200;

        // Create a channel to get the result
        let (otx, orx) = oneshot::channel();

        // Send our request to the processing task
        tx.send((url, browser, otx)).await.unwrap();

        // Spawn a task to wait for the processing result
        let task = tokio::spawn(async move {
            sleep(Duration::from_millis(random_delay)).await;

            let _permit = sem_clone.acquire().await.unwrap();

            // Wait for the processing result
            orx.await.unwrap();
        });

        tasks.spawn(task);
    }

    handle_task_results(tasks).await;

    Ok(())
}

async fn process(mut rx: mpsc::Receiver<(String, Arc<Mutex<Browser>>, oneshot::Sender<()>)>) {
    // Receive the next queued request
    while let Some((url, browser, tx)) = rx.recv().await {
        // Process the request
        println!("URL {}", &url );

        if let Ok(page) = browser.lock().unwrap().new_tab() {
            if let Ok(tab) = page.navigate_to(&url) {
                if tab.wait_for_element(".show-more-less-html__markup").is_ok() {
                    if let Ok(element) = tab.find_element(".show-more-less-html__markup") {
                        println!("{}", &url);
                        let content = element.get_content().unwrap();
                        println!("{}", content);
                    }
                }
            }
        }

        // Send back the result
        tx.send(()).unwrap();
    }
}

async fn handle_task_results(mut tasks: JoinSet<Result<(), JoinError>>) {
    println!("Waiting for all tasks to complete ...\n");
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(_)) => {
                // The task completed successfully
            }
            Ok(Err(e)) => {
                // The task returned an error
                eprintln!("Task returned an error: {:?}", e);
            }
            Err(e) => {
                // The task was cancelled
                eprintln!("Task was cancelled: {:?}", e);
            }
        }
    }
}
use headless_chrome::Browser;
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{sleep, Duration};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct Job {
    url: String,
    body: String,
    salary: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a semaphore maximum of 4 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(4));
    let mut tasks: JoinSet<Result<(), JoinError>> = JoinSet::new();
    let jobs = Arc::new(std::sync::RwLock::new(Vec::new()));
    let contents = std::fs::read_to_string("small_json_test.csv")?;
    let urls: Vec<&str> = contents.split(",").collect();

    //allows Us to share the browser across multiple tasks.
    let browser = Arc::new(Mutex::new(Browser::default().unwrap()));

    urls.into_iter().for_each(|url| {
        let sem_clone = Arc::clone(&semaphore);
        let url = url.to_owned();

        println!("----- owned url {}", url);

        let browser = Arc::clone(&browser);
        let random_delay: u64 = rand::thread_rng().gen_range(50..=80) + 100;
        let jobs = Arc::clone(&jobs);

        let task = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            println!("URL {}\nDelay {} ms", &url, random_delay);

            if let Ok(page) = browser.lock().unwrap().new_tab() {
                if let Ok(tab) = page.navigate_to(&url) {
                    tab.wait_for_xpath_with_custom_timeout(
                        ".show-more-less-html__markup",
                        Duration::from_millis(250),
                    )
                    .map(|elm| {
                        println!("Found element");
                        let content = elm.get_content().unwrap();
                        println!("{}", content);

                        let salary: Option<String> = content
                            .find("Salary:")
                            .map(|index| content[index..].to_string());

                        let job = Job {
                            url: url.clone(),
                            body: content,
                            salary,
                        };

                        let mut jobs_write = jobs.write().unwrap();
                        jobs_write.push(job);
                    })
                    .expect("Failed to find element on");
                    // 
                    /* if we .unwrap(); here we see JoinError::Panic(Id(23), ...)
                    
                    thread 'tokio-runtime-worker' panicked at src\main.rs:67:22:
                    called `Result::unwrap()` on an `Err` value: Method call error -32000: No search session with given id found
                    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
                    thread 'Task returned an error: tokio-runtime-workerJoinError::Panic(' panicked at Idsrc\main.rs(:2143):, ...)46
                    :
                    called `Result::unwrap()` on an `Err` value: PoisonError { .. }
                    thread 'Task returned an error: tokio-runtime-workerJoinError::Panic(' panicked at Idsrc\main.rs(:1743):, ...)46
                    :
                    called `Result::unwrap()` on an `Err` value: PoisonError { .. }
                    Task returned an error: JoinError::Panic(Id(23), ...)
                    Outputed Jobs to export_jobs.json
                     */
                }
            }
            sleep(Duration::from_millis(random_delay)).await;
        });

        tasks.spawn(task);
    });

    handle_task_results(tasks).await;

    let file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("export_jobs.json")
        .unwrap();

    let json_out = jobs.read().unwrap();
    serde_json::to_writer(file, &*json_out)
        .map(|_| println!("Outputed Jobs to export_jobs.json"))
        .expect("Error writing to file");

    Ok(())
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
